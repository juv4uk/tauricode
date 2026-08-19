//! Orchestrates one scan of the configured root: one `scan-id` for the
//! whole `EcosystemSnapshot`, per-repository independence (one repo
//! failing never removes another repo's data), per
//! `ECO-DECISION-2026-08-19-TAURICODE-STAGE1-OBSERVER`.

use crate::git_read::{self, RepoKind};
use crate::identity_contract;
use crate::process_observe::{self, OsProcess};
use crate::snapshot::{
    AgentProcess, EcosystemSnapshot, GitState, IdentityStatus, OsObservedFacts, ProbeFailure,
    RepositorySnapshot, ScanMetadata, ScanStatus, SelfReportedIdentity,
};
use crate::time_util::iso8601_now;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct DiscoverInput {
    /// No default baked into the crate — the caller (this environment's
    /// coordinator tooling, a future Tauri command, a test) supplies it.
    pub root: PathBuf,
    /// `None` = auto-discover every subdirectory of `root`; `Some` scans
    /// exactly the named subdirectories, in the given order.
    pub repositories: Option<Vec<String>>,
    /// `None` = use `identity_contract::default_base_dir()` (the real
    /// `$XDG_RUNTIME_DIR/ecosystem-agents` location). `Some(path)` reads
    /// self-reported identity records from exactly that directory instead
    /// — for tests, so they never depend on (or race on) a real shared
    /// runtime directory; production callers should leave this `None`.
    pub identity_base_dir: Option<PathBuf>,
}

pub fn discover_ecosystem(input: DiscoverInput) -> EcosystemSnapshot {
    let scan_id = generate_scan_id();
    let started_at = iso8601_now();

    let targets = resolve_targets(&input);
    let repositories: Vec<RepositorySnapshot> = targets
        .into_iter()
        .map(|(name, path)| scan_repository(&name, &path))
        .collect();

    let identity_base_dir = input
        .identity_base_dir
        .unwrap_or_else(identity_contract::default_base_dir);
    let local_processes = gather_local_processes(&repositories, &identity_base_dir);

    let finished_at = iso8601_now();
    EcosystemSnapshot {
        scan: ScanMetadata {
            scan_id,
            started_at,
            finished_at,
        },
        repositories,
        local_processes,
    }
}

/// Slice 2, two passes over two different sources of truth — never
/// merged into one loop, because they answer different questions:
///
/// 1. Live OS processes (from `process_observe::list_processes()`) that
///    are relevant — cwd inside one of this scan's repositories, or a
///    self-reported record exists for that PID — get `os_observed:
///    Some(...)`, correlated against any self-reported record.
/// 2. Self-reported PIDs that have **no** matching live OS process at
///    all get `os_observed: None`, `identity_status: Orphaned` — the
///    identity file is real, but nothing backs its PID claim right now.
///    This is what makes an orphaned file for a dead process visible
///    instead of silently dropped (found in adversarial review of the
///    first version of this function, which only ever did pass 1).
fn gather_local_processes(
    repositories: &[RepositorySnapshot],
    identity_base_dir: &Path,
) -> Vec<AgentProcess> {
    let known_pids = identity_contract::known_pids(identity_base_dir);
    let now = SystemTime::now();
    let live_processes = process_observe::list_processes();
    let live_pids: HashSet<u32> = live_processes.iter().map(|p| p.pid).collect();

    let mut result: Vec<AgentProcess> = live_processes
        .into_iter()
        .filter_map(|process| {
            build_live_agent_process(process, repositories, &known_pids, identity_base_dir, now)
        })
        .collect();

    for pid in known_pids {
        if live_pids.contains(&pid) {
            continue; // already handled in pass 1, with real correlation
        }
        result.push(AgentProcess {
            pid,
            os_observed: None,
            identity_status: IdentityStatus::Orphaned,
            identity: SelfReportedIdentity::default(),
        });
    }

    result
}

fn build_live_agent_process(
    process: OsProcess,
    repositories: &[RepositorySnapshot],
    known_pids: &HashSet<u32>,
    base_dir: &Path,
    now: SystemTime,
) -> Option<AgentProcess> {
    let repo_association = process
        .cwd
        .as_deref()
        .and_then(|cwd| find_repo_association(cwd, repositories));

    let is_relevant = repo_association.is_some() || known_pids.contains(&process.pid);
    if !is_relevant {
        return None;
    }

    let (identity_status, identity) = identity_contract::correlate(&process, base_dir, now);

    Some(AgentProcess {
        pid: process.pid,
        os_observed: Some(OsObservedFacts {
            command: process.command,
            cwd: process.cwd,
            started_at_observed: process.started_at,
            repo_association,
        }),
        identity_status,
        identity,
    })
}

/// A process's `cwd` is associated with the first repository (in scan
/// order) whose path is a prefix of it. Plain path-prefix comparison, not
/// canonicalized on either side — matches how this crate's own callers
/// have used it all session (direct, non-symlinked absolute paths); a
/// symlinked repo path could defeat this, a known, undocumented-until-now
/// limitation, not fixed here.
fn find_repo_association(cwd: &str, repositories: &[RepositorySnapshot]) -> Option<String> {
    let cwd_path = Path::new(cwd);
    repositories
        .iter()
        .find(|repo| cwd_path.starts_with(&repo.path))
        .map(|repo| repo.name.clone())
}

/// Scans one repository. Never panics on a bad/missing/non-git path —
/// degrades to `ScanStatus::Failed` with an `error` string instead, so one
/// repository's problem never removes the rest of the snapshot's data.
///
/// Repository identity is checked first via
/// [`git_read::identify_repository`], which distinguishes three cases
/// (this is a hardening fix — Slice 1's first commit used a check that
/// silently reported an *ancestor* repository's state for any nested
/// non-git directory; see that function's doc comment):
/// - not a repository root at this exact path → `Failed`, `git: None`.
///   This covers both "no git repository anywhere in reach" and "inside a
///   git working tree, but not that repository's own root" (an
///   uninitialized submodule, a plain subdirectory, ...) — both produce an
///   explicit, distinct `error` string, never inherited data.
/// - a valid bare repository → `Failed`, `git: None`, with an `error` that
///   says so explicitly (never "not a git repository" — a bare repo *is*
///   one). Slice 1's `GitState` models working-tree fields (dirty-state,
///   checked-out branch) that don't apply to a bare repository; supporting
///   bare repositories properly is future scope, not silently wrong data.
/// - a working tree (ordinary repository or linked worktree — both have
///   their own `--show-toplevel`) → proceed to the four read-only probes.
///
/// Within a working tree, the four probes (branch, head SHA, dirty-state,
/// remotes) are evaluated independently:
/// - all four succeed → `Complete`, full `GitState`.
/// - every probe fails (including via timeout — see
///   `git_read::run_with_timeout`) → `Failed`, `git: None` (repository
///   state could not be obtained at all, even though it is a repository).
/// - some succeed and some fail → `Partial`, `GitState` holds the facts
///   that were read plus an `unavailable` list naming which probes failed
///   and why. Nothing is silently defaulted without that failure being
///   recorded.
pub fn scan_repository(name: &str, path: &Path) -> RepositorySnapshot {
    let path_str = path.display().to_string();

    if !path.exists() {
        let reason = if path.symlink_metadata().is_ok() {
            "broken symlink (target does not exist)"
        } else {
            "path does not exist"
        };
        return RepositorySnapshot {
            name: name.to_string(),
            path: path_str,
            scan_status: ScanStatus::Failed,
            git: None,
            error: Some(reason.to_string()),
        };
    }

    match git_read::identify_repository(path) {
        Ok(RepoKind::NotARepository { reason }) => {
            return RepositorySnapshot {
                name: name.to_string(),
                path: path_str,
                scan_status: ScanStatus::Failed,
                git: None,
                error: Some(reason.to_string()),
            };
        }
        Ok(RepoKind::Bare) => {
            return RepositorySnapshot {
                name: name.to_string(),
                path: path_str,
                scan_status: ScanStatus::Failed,
                git: None,
                error: Some(
                    "bare repository (no working tree) — this IS a git repository, \
                     but Slice 1's GitState models working-tree fields (checked-out \
                     branch, dirty-state) that don't apply to it; unsupported here, \
                     not invalid or missing"
                        .to_string(),
                ),
            };
        }
        Ok(RepoKind::WorkingTree) => {} // fall through to the four probes below
        Err(reason) => {
            return RepositorySnapshot {
                name: name.to_string(),
                path: path_str,
                scan_status: ScanStatus::Failed,
                git: None,
                error: Some(format!("failed to determine repository kind: {reason}")),
            };
        }
    }

    let branch_result = git_read::read_branch(path);
    let head_result = git_read::read_head_sha(path);
    let dirty_result = git_read::read_dirty_state(path);
    let remotes_result = git_read::read_remotes(path);

    let mut unavailable = Vec::new();

    let (branch, is_detached) = match branch_result {
        Ok(v) => v,
        Err(reason) => {
            unavailable.push(ProbeFailure {
                probe: "branch".to_string(),
                reason,
            });
            (None, false)
        }
    };
    let head_sha = match head_result {
        Ok(v) => v,
        Err(reason) => {
            unavailable.push(ProbeFailure {
                probe: "head_sha".to_string(),
                reason,
            });
            None
        }
    };
    let (is_dirty, changed_paths) = match dirty_result {
        Ok(v) => v,
        Err(reason) => {
            unavailable.push(ProbeFailure {
                probe: "dirty".to_string(),
                reason,
            });
            (false, Vec::new())
        }
    };
    let remotes = match remotes_result {
        Ok(v) => v,
        Err(reason) => {
            unavailable.push(ProbeFailure {
                probe: "remotes".to_string(),
                reason,
            });
            Vec::new()
        }
    };

    const PROBE_COUNT: usize = 4;
    if unavailable.len() == PROBE_COUNT {
        // Every probe failed — the path is a git repository (the initial
        // gate passed) but its state could not be obtained at all. That's
        // "repository state entirely unobtainable", i.e. `Failed`, not a
        // `GitState` with four placeholder fields and nothing real in it.
        return RepositorySnapshot {
            name: name.to_string(),
            path: path_str,
            scan_status: ScanStatus::Failed,
            git: None,
            error: Some(summarize(&unavailable)),
        };
    }

    let scan_status = if unavailable.is_empty() {
        ScanStatus::Complete
    } else {
        ScanStatus::Partial
    };
    let error = if unavailable.is_empty() {
        None
    } else {
        Some(summarize(&unavailable))
    };

    RepositorySnapshot {
        name: name.to_string(),
        path: path_str,
        scan_status,
        git: Some(GitState {
            branch,
            is_detached,
            head_sha,
            is_dirty,
            changed_paths,
            remotes,
            unavailable,
        }),
        error,
    }
}

fn summarize(failures: &[ProbeFailure]) -> String {
    failures
        .iter()
        .map(|f| format!("{}: {}", f.probe, f.reason))
        .collect::<Vec<_>>()
        .join("; ")
}

fn resolve_targets(input: &DiscoverInput) -> Vec<(String, PathBuf)> {
    match &input.repositories {
        Some(names) => names
            .iter()
            .map(|n| (n.clone(), input.root.join(n)))
            .collect(),
        None => list_subdirectories(&input.root),
    }
}

/// Every subdirectory of `root`, git or not, plus every symlink (even a
/// broken one) — non-git directories and broken symlinks are not filtered
/// out here; they surface later as `ScanStatus::Failed` entries with an
/// explicit `error`, per Stage 1's "show unknown/failure, don't hide it"
/// principle, rather than being silently excluded from the snapshot.
///
/// This is a hardening fix: `path.is_dir()` alone follows symlinks and
/// returns `false` for a broken one exactly the same as for an ordinary
/// non-directory file, which meant a broken symlink used to vanish from
/// the snapshot entirely — the one case this function's own doc comment
/// already claimed didn't happen. `DirEntry::file_type()` (which does
/// *not* follow symlinks) is used to tell "broken symlink" apart from
/// "ordinary file", so only the former is still included.
fn list_subdirectories(root: &Path) -> Vec<(String, PathBuf)> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let is_symlink = entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false);
            if path.is_dir() || is_symlink {
                result.push((name.to_string(), path));
            }
        }
    }
    result.sort_by(|a, b| a.0.cmp(&b.0)); // deterministic order, mainly for tests
    result
}

fn generate_scan_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("scan-{nanos}-{}", std::process::id())
}
