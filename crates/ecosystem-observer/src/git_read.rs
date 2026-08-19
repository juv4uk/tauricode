//! Read-only git plumbing. Every function shells out to the real `git`
//! binary via `std::process::Command` — no `git2`/`gix` dependency. This
//! matches the convention already used by `my-idea`'s `ecosystem::git`
//! module and by tauricode's own (Electron-era) WSL runtime code.
//!
//! No command here can mutate repository state: `--no-optional-locks` and
//! the read-only subcommands used (`rev-parse`, `symbolic-ref`, `status`,
//! `remote -v`) never write refs, objects, or the index. Verified
//! empirically (not just asserted): plain `git status` touches
//! `.git/index`'s mtime, `git --no-optional-locks status` does not.
//!
//! Every invocation goes through [`run_with_timeout`], so a wedged git
//! process (stale lock, a hanging `include.path`, a stalled
//! network-mounted filesystem) fails that one probe instead of hanging
//! the whole scan indefinitely.

use crate::snapshot::RemoteInfo;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

/// Safety flags copied from `packages/opencode/src/git/index.ts` (tauricode,
/// `origin/dev`) — avoids optional-lock contention and cross-platform
/// autocrlf/symlink/longpaths/quotepath surprises during read-only git
/// invocations. Copied as a value, not imported: importing that module
/// would pull in `effect`/`LayerNode`/`AppProcess` for a constant array.
const GIT_SAFETY_FLAGS: &[&str] = &[
    "--no-optional-locks",
    "-c",
    "core.autocrlf=false",
    "-c",
    "core.fsmonitor=false",
    "-c",
    "core.longpaths=true",
    "-c",
    "core.symlinks=true",
    "-c",
    "core.quotepath=false",
];

/// Conservative default: long enough for a slow cold start (network-mounted
/// filesystem, cold page cache on a large repo), short enough to fail a
/// wedged probe rather than hang the whole scan indefinitely.
const GIT_PROBE_TIMEOUT: Duration = Duration::from_secs(20);

/// Runs `command` to completion, or kills it and returns `Err` if it
/// hasn't exited within `timeout`.
///
/// Stdout/stderr are drained concurrently on separate threads while
/// waiting. This isn't optional: piping without doing this can itself
/// deadlock a child that writes more than the OS pipe buffer holds while
/// nothing is reading — which would silently reintroduce a hang this
/// function exists to prevent, just for large-output repos instead of
/// wedged ones.
fn run_with_timeout(mut command: Command, timeout: Duration) -> Result<Output, String> {
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|e| format!("failed to spawn process: {e}"))?;

    let mut stdout_pipe = child.stdout.take().expect("stdout was piped");
    let mut stderr_pipe = child.stderr.take().expect("stderr was piped");
    let stdout_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buf);
        buf
    });
    let stderr_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buf);
        buf
    });

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait(); // reap, avoid a zombie
                    let _ = stdout_thread.join();
                    let _ = stderr_thread.join();
                    return Err(format!(
                        "process timed out after {timeout:?} and was killed"
                    ));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                let _ = stdout_thread.join();
                let _ = stderr_thread.join();
                return Err(format!("failed to poll process: {e}"));
            }
        }
    };

    let stdout = stdout_thread.join().unwrap_or_default();
    let stderr = stderr_thread.join().unwrap_or_default();
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

fn run_git(path: &Path, args: &[&str]) -> Result<Output, String> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(path)
        .args(GIT_SAFETY_FLAGS)
        .args(args);
    run_with_timeout(command, GIT_PROBE_TIMEOUT)
}

fn stdout_trimmed(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// How [`identify_repository`] classifies a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoKind {
    /// A repository with a working tree, rooted exactly at the queried
    /// path. Covers ordinary repositories and linked worktrees alike —
    /// both have their own `--show-toplevel` equal to their own path.
    WorkingTree,
    /// A valid bare repository (no working tree), rooted exactly at the
    /// queried path.
    Bare,
    /// Not a repository root at this exact path — either genuinely not a
    /// git repository at all, or a path that is *inside* one without
    /// being that repository's own root (an uninitialized submodule
    /// directory, an ordinary subdirectory of a working tree, a broken
    /// symlink, ...).
    NotARepository { reason: &'static str },
}

/// Classifies `path` without assuming it's a repository root just because
/// git's own upward directory search found a `.git` somewhere above it.
///
/// This is the fix for a real bug found in adversarial review of Slice 1:
/// `git -C <path> rev-parse --is-inside-work-tree` succeeds — reporting
/// the *ancestor* repository's state — for any plain or uninitialized
/// subdirectory nested inside a git working tree, because `-C` does not
/// restrict git's own directory discovery to stop at `<path>`. The fix is
/// to additionally require that git's own idea of the repository's root
/// (`--show-toplevel` for a working tree, `--absolute-git-dir` for a bare
/// repository) resolves to the *same* canonicalized path as the one being
/// queried — not merely to be inside one somewhere.
pub fn identify_repository(path: &Path) -> Result<RepoKind, String> {
    let target_canonical = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => {
            return Ok(RepoKind::NotARepository {
                reason: "path does not exist or is not readable",
            })
        }
    };

    let bare_output = run_git(path, &["rev-parse", "--is-bare-repository"])?;
    if !bare_output.status.success() {
        return Ok(RepoKind::NotARepository {
            reason: "no git repository found at this path or in any ancestor directory",
        });
    }
    let is_bare = stdout_trimmed(&bare_output) == "true";

    if is_bare {
        let gitdir_output = run_git(path, &["rev-parse", "--absolute-git-dir"])?;
        if !gitdir_output.status.success() {
            return Ok(RepoKind::NotARepository {
                reason: "git reported this as a bare repository but --absolute-git-dir failed",
            });
        }
        let gitdir = stdout_trimmed(&gitdir_output);
        return Ok(match std::fs::canonicalize(&gitdir) {
            Ok(gitdir_canonical) if gitdir_canonical == target_canonical => RepoKind::Bare,
            _ => RepoKind::NotARepository {
                reason: "a bare git directory was found, but above this path, not at it",
            },
        });
    }

    let toplevel_output = run_git(path, &["rev-parse", "--show-toplevel"])?;
    if !toplevel_output.status.success() {
        return Ok(RepoKind::NotARepository {
            reason: "git repository state could not be determined (--show-toplevel failed)",
        });
    }
    let toplevel = stdout_trimmed(&toplevel_output);
    Ok(match std::fs::canonicalize(&toplevel) {
        Ok(toplevel_canonical) if toplevel_canonical == target_canonical => RepoKind::WorkingTree,
        _ => RepoKind::NotARepository {
            reason: "this path is inside a git working tree, but is not that repository's own root (e.g. an uninitialized submodule, or a plain subdirectory)",
        },
    })
}

/// `(branch, is_detached)`. `symbolic-ref` fails exactly when HEAD is not a
/// symbolic ref, i.e. detached — that specific failure is not an error for
/// our purposes, it's the detached-HEAD signal itself.
pub fn read_branch(path: &Path) -> Result<(Option<String>, bool), String> {
    let output = run_git(path, &["symbolic-ref", "--short", "HEAD"])?;
    if output.status.success() {
        Ok((Some(stdout_trimmed(&output)), false))
    } else {
        Ok((None, true))
    }
}

/// `None` only for a repository with no commits yet (unborn branch) — a
/// normal, successful outcome, not a read failure.
pub fn read_head_sha(path: &Path) -> Result<Option<String>, String> {
    let output = run_git(path, &["rev-parse", "HEAD"])?;
    if output.status.success() {
        Ok(Some(stdout_trimmed(&output)))
    } else {
        Ok(None)
    }
}

/// `(is_dirty, changed_paths)` via `git status --porcelain=v1 -z`.
///
/// Field order for rename entries was verified empirically (not from
/// memory) against git 2.43.0: `git status --porcelain=v1 -z` emits
/// `XY <new-path>\0<old-path>\0`, i.e. the *new* path first, then the old
/// one — the reverse of what a naive reading of "old -> new" might suggest.
/// See `rename_and_modify_is_reported_as_old_arrow_new` and
/// `pure_rename_without_content_change_is_also_reported` in
/// `tests/discover_tests.rs`. Also verified against unstaged renames: a
/// plain filesystem `mv` without `git add` produces two separate entries
/// (a deletion and an untracked addition), not a combined rename — this
/// function only combines what git's own status output already combines
/// (staged renames), it does not detect renames on its own.
///
/// The `code.contains('C')` branch below handles copy entries the same
/// way, on the assumption that git's `-z` copy format matches its rename
/// format (both use the two-path `R`/`C` status-code shape per
/// git-status(1)). **That assumption is unverified**: `git status` does
/// not detect copies unless invoked with `--find-copies`/`-C`, which this
/// function deliberately does not pass (copy detection is off by default
/// and more expensive to compute) — so in practice, with the invocation
/// used here, git never emits a `C` status code and this branch is
/// effectively dead code today. Treat copy handling as unverified, not as
/// a tested capability, until someone has an actual reason to enable
/// `--find-copies` and add a corresponding test.
pub fn read_dirty_state(path: &Path) -> Result<(bool, Vec<String>), String> {
    let output = run_git(path, &["status", "--porcelain=v1", "-z"])?;
    if !output.status.success() {
        return Err(format!(
            "git status failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let mut fields: Vec<String> = output
        .stdout
        .split(|&b| b == 0)
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect();
    // trailing NUL produces one empty field at the end; drop it.
    if fields.last().map(|s| s.is_empty()).unwrap_or(false) {
        fields.pop();
    }

    let mut changed = Vec::new();
    let mut iter = fields.drain(..);
    while let Some(entry) = iter.next() {
        if entry.len() < 3 {
            continue; // malformed/short entry — defensively skip, don't panic
        }
        let code = &entry[0..2];
        let new_path = entry[3..].to_string();
        let is_rename_or_copy = code.contains('R') || code.contains('C');
        if is_rename_or_copy {
            match iter.next() {
                Some(old_path) => changed.push(format!("{old_path} -> {new_path}")),
                None => changed.push(new_path), // malformed stream, degrade gracefully
            }
        } else {
            changed.push(new_path);
        }
    }

    Ok((!changed.is_empty(), changed))
}

/// Observed remotes via `git remote -v`, deduplicated by (name, url) — a
/// remote with identical fetch/push URLs (the common case) yields one
/// entry, not two.
pub fn read_remotes(path: &Path) -> Result<Vec<RemoteInfo>, String> {
    let output = run_git(path, &["remote", "-v"])?;
    if !output.status.success() {
        return Err(format!(
            "git remote -v failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut remotes: Vec<RemoteInfo> = Vec::new();
    for line in text.lines() {
        let mut parts = line.splitn(2, '\t');
        let name = match parts.next() {
            Some(n) if !n.is_empty() => n.to_string(),
            _ => continue,
        };
        let rest = match parts.next() {
            Some(r) => r,
            None => continue,
        };
        let url = rest
            .rsplit_once(' ')
            .map(|(u, _)| u)
            .unwrap_or(rest)
            .to_string();
        let entry = RemoteInfo { name, url };
        if !remotes.contains(&entry) {
            remotes.push(entry);
        }
    }
    Ok(remotes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Proves the timeout mechanism itself: a process that outlives its
    /// deadline is killed and reported as an error quickly, not blocked
    /// on. Uses a generic `sleep`, not git — the git-specific timeout
    /// regression (a real wedged `git` invocation, via a hanging
    /// `include.path`) lives in `tests/discover_tests.rs` since it
    /// exercises the full `scan_repository` path, at the cost of running
    /// for real time (~20s) rather than a few hundred milliseconds.
    #[test]
    fn run_with_timeout_kills_a_hung_process_instead_of_blocking() {
        let mut cmd = Command::new("sleep");
        cmd.arg("5");
        let started = Instant::now();
        let result = run_with_timeout(cmd, Duration::from_millis(200));
        let elapsed = started.elapsed();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timed out"));
        assert!(
            elapsed < Duration::from_secs(2),
            "should return shortly after the deadline, took {elapsed:?}"
        );
    }

    #[test]
    fn run_with_timeout_returns_normal_output_for_fast_commands() {
        let mut cmd = Command::new("echo");
        cmd.arg("hello");
        let result = run_with_timeout(cmd, Duration::from_secs(5)).unwrap();
        assert!(result.status.success());
        assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), "hello");
    }
}
