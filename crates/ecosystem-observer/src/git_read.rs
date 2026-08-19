//! Read-only git plumbing. Every function shells out to the real `git`
//! binary via `std::process::Command` — no `git2`/`gix` dependency. This
//! matches the convention already used by `my-idea`'s `ecosystem::git`
//! module and by tauricode's own (Electron-era) WSL runtime code.
//!
//! No command here can mutate repository state: `--no-optional-locks` and
//! the read-only subcommands used (`rev-parse`, `symbolic-ref`, `status`,
//! `remote -v`) never write refs, objects, or the index.

use crate::snapshot::RemoteInfo;
use std::path::Path;
use std::process::{Command, Output};

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

fn run_git(path: &Path, args: &[&str]) -> Result<Output, String> {
    Command::new("git")
        .arg("-C")
        .arg(path)
        .args(GIT_SAFETY_FLAGS)
        .args(args)
        .output()
        .map_err(|e| format!("failed to spawn git: {e}"))
}

fn stdout_trimmed(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// `true` iff `path` is (or is inside) a git working tree. Used as the
/// gate before running any of the other read functions.
pub fn is_git_repo(path: &Path) -> bool {
    match run_git(path, &["rev-parse", "--is-inside-work-tree"]) {
        Ok(output) => output.status.success() && stdout_trimmed(&output) == "true",
        Err(_) => false,
    }
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
/// `tests/discover_tests.rs`.
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
