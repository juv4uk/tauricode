//! Data model for Slice 1, per `ECO-DECISION-2026-08-19-TAURICODE-STAGE1-OBSERVER`.
//! `EcosystemSnapshot`/`RepositorySnapshot`/`GitState`/`ScanMetadata` only —
//! no Guix, tasks, or evidence fields on this slice.

/// `Complete`: the repository is a git repo and all four read-only probes
/// (branch, head SHA, dirty-state, remotes) succeeded.
/// `Partial`: it is a git repo, but one or more probes failed while at
/// least one other succeeded — `GitState` holds the facts that *were*
/// read, and `GitState::unavailable` names which ones weren't.
/// `Failed`: repository state could not be obtained at all — either the
/// path isn't a git repository, or every probe failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanStatus {
    Complete,
    Partial,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
}

/// One read-only probe (`"branch"`, `"head_sha"`, `"dirty"`, or
/// `"remotes"` — matching the function names in `git_read`) that failed
/// for an otherwise-valid git repository, with a human-readable reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeFailure {
    pub probe: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitState {
    /// Placeholder (`None`) when the `"branch"` probe is in `unavailable`
    /// — check `unavailable` before treating this as "detached", which is
    /// its other, legitimate meaning.
    pub branch: Option<String>,
    /// Meaningless (`false`) when the `"branch"` probe is in
    /// `unavailable` — the pair (`branch`, `is_detached`) is only trustworthy
    /// as a whole when that probe succeeded.
    pub is_detached: bool,
    /// `None` either because the repository has no commits yet (unborn
    /// branch — a normal, successful outcome) or because the `"head_sha"`
    /// probe is in `unavailable` (a failure). Check `unavailable` to tell
    /// the two apart.
    pub head_sha: Option<String>,
    /// Placeholder (`false`) when the `"dirty"` probe is in `unavailable`.
    pub is_dirty: bool,
    /// One entry per changed path. Rename entries are rendered as
    /// `"old -> new"` (matches git's own non-`-z` display), not split into
    /// two separate paths. Empty placeholder when the `"dirty"` probe is
    /// in `unavailable` — check `unavailable`, an empty list here is not
    /// on its own evidence of a clean tree in that case.
    ///
    /// Copy entries (`git status`'s `C` status code) are handled by the
    /// same code path as renames if they ever occur, but this crate does
    /// not pass `--find-copies`/`-C` to `git status` (copy detection is
    /// off by default and off here), so that branch is currently
    /// unverified/effectively unreachable in practice — see
    /// `git_read::read_dirty_state`'s doc comment. Treat "copy" support
    /// as unverified, not as a tested capability.
    pub changed_paths: Vec<String>,
    /// Observed as-is. No "expected remote" comparison exists yet in any
    /// ecosystem contract, so there is nothing to validate against.
    /// Empty placeholder when the `"remotes"` probe is in `unavailable` —
    /// check `unavailable`, an empty list here is not on its own evidence
    /// of "no remotes configured" in that case.
    pub remotes: Vec<RemoteInfo>,
    /// Which of the four probes above failed, if any. Empty iff
    /// `RepositorySnapshot::scan_status` is `Complete`.
    pub unavailable: Vec<ProbeFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositorySnapshot {
    pub name: String,
    pub path: String,
    pub scan_status: ScanStatus,
    pub git: Option<GitState>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanMetadata {
    pub scan_id: String,
    pub started_at: String,
    pub finished_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcosystemSnapshot {
    pub scan: ScanMetadata,
    pub repositories: Vec<RepositorySnapshot>,
}
