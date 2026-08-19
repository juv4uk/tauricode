//! Orchestrates one scan of the configured root: one `scan-id` for the
//! whole `EcosystemSnapshot`, per-repository independence (one repo
//! failing never removes another repo's data), per
//! `ECO-DECISION-2026-08-19-TAURICODE-STAGE1-OBSERVER`.

use crate::git_read::{self, RepoKind};
use crate::snapshot::{
    EcosystemSnapshot, GitState, ProbeFailure, RepositorySnapshot, ScanMetadata, ScanStatus,
};
use crate::time_util::iso8601_now;
use std::path::{Path, PathBuf};

pub struct DiscoverInput {
    /// No default baked into the crate — the caller (this environment's
    /// coordinator tooling, a future Tauri command, a test) supplies it.
    pub root: PathBuf,
    /// `None` = auto-discover every subdirectory of `root`; `Some` scans
    /// exactly the named subdirectories, in the given order.
    pub repositories: Option<Vec<String>>,
}

pub fn discover_ecosystem(input: DiscoverInput) -> EcosystemSnapshot {
    let scan_id = generate_scan_id();
    let started_at = iso8601_now();

    let targets = resolve_targets(&input);
    let repositories = targets
        .into_iter()
        .map(|(name, path)| scan_repository(&name, &path))
        .collect();

    let finished_at = iso8601_now();
    EcosystemSnapshot {
        scan: ScanMetadata {
            scan_id,
            started_at,
            finished_at,
        },
        repositories,
    }
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
