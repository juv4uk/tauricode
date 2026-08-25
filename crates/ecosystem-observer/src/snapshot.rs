//! Data model for Slice 1, per `ECO-DECISION-2026-08-19-TAURICODE-STAGE1-OBSERVER`.
//! `EcosystemSnapshot`/`RepositorySnapshot`/`GitState`/`ScanMetadata` only —
//! no Guix, tasks, or evidence fields on this slice.
//!
//! `serde::Serialize` added to every type here (2026-08-25, first real
//! `packages/desktop-tauri/` slice per `ECO-DECISION-2026-08-19-
//! TAURICODE-TAURI-ARCHITECTURE`): a Tauri command needs to return this
//! shape as JSON to the frontend. `serde`/`serde_json` were already a
//! dependency of this crate (Slice 2's identity-contract parsing), so
//! this is a derive addition, not a new dependency. `Deserialize` is
//! deliberately not added — nothing in this crate or its consumers
//! reads an `EcosystemSnapshot` back in, only produces one.
//!
//! **Known technical debt, recorded deliberately rather than fixed here:**
//! `GitState`'s per-probe fields (`is_dirty: bool`, `changed_paths:
//! Vec<String>`, `remotes: Vec<RemoteInfo>`, `branch: Option<String>`) are
//! plain, ordinary-looking types. During `ScanStatus::Partial`, a field
//! whose probe is listed in `unavailable` holds a placeholder (`false`,
//! empty, or `None`), not a fact — but nothing at the type level stops a
//! consumer from reading e.g. `git.is_dirty` as "definitely clean" without
//! first checking whether `"dirty"` is in `git.unavailable`. This was
//! flagged in adversarial review of the first commit and deliberately left
//! as-is per explicit instruction not to change these field shapes in
//! this hardening pass. A future slice should consider a typed wrapper
//! (e.g. an enum per field distinguishing "known" from "probe failed")
//! instead of bare `bool`/`Vec` — not done now to avoid an API change
//! outside this hardening's authorized scope.

/// `Complete`: the repository is a git repo and all four read-only probes
/// (branch, head SHA, dirty-state, remotes) succeeded.
/// `Partial`: it is a git repo, but one or more probes failed while at
/// least one other succeeded — `GitState` holds the facts that *were*
/// read, and `GitState::unavailable` names which ones weren't.
/// `Failed`: repository state could not be obtained at all — either the
/// path isn't a git repository, or every probe failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ScanStatus {
    Complete,
    Partial,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
}

/// One read-only probe (`"branch"`, `"head_sha"`, `"dirty"`, or
/// `"remotes"` — matching the function names in `git_read`) that failed
/// for an otherwise-valid git repository, with a human-readable reason.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProbeFailure {
    pub probe: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RepositorySnapshot {
    pub name: String,
    pub path: String,
    pub scan_status: ScanStatus,
    pub git: Option<GitState>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ScanMetadata {
    pub scan_id: String,
    pub started_at: String,
    pub finished_at: String,
}

/// Slice 2 (`ECO-DECISION` follow-up, "Local Runtime Observation + Agent
/// Identity Contract"): freshness/state of a self-reported identity
/// record, evaluated by correlating it against the OS-observed process it
/// claims to describe. This is a *separate axis* from provenance
/// (OS-observed vs. self-reported) — it is not itself a provenance value.
///
/// **This is a correlation/liveness status, not a trust/authenticity
/// status.** `Fresh` means "a record exists whose claimed PID and start
/// time line up with a real, currently-running process, and it was
/// updated recently" — it does *not* mean "this record was written by
/// the process it claims to describe" or "this record's content is
/// true". Nothing in this crate verifies *who* wrote a self-reported
/// file, only *whether its PID claim is currently consistent with
/// reality*. See `identity_contract`'s module doc for the known gap this
/// leaves open (the runtime directory may be group-writable).
///
/// - `Fresh`: a self-reported record exists, its PID correlates with a
///   real running OS-observed process (via `process_start_token`, or the
///   `started_at` tolerance fallback — see `identity_contract`), and its
///   `updated_at` is within the staleness threshold.
/// - `Stale`: correlates correctly, but `updated_at` is older than the
///   threshold — the process is real and is who it claims to be, but may
///   not have reported in for a while.
/// - `Orphaned`: a self-reported record's PID does not correlate with the
///   OS-observed process — either that PID belongs to a *different*
///   process now (PID reuse, caught by `process_start_token`/tolerance
///   mismatch) or, when reached via that path, no such process exists at
///   all. Its identity fields are never surfaced as this process's own.
/// - `NotFound`: the OS-observed process is real (and relevant — see
///   `AgentProcess`) but has no self-reported record at all. Not an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum IdentityStatus {
    Fresh,
    Stale,
    Orphaned,
    NotFound,
}

/// Everything in this struct is provenance: self-reported. None of it is
/// verified against anything except the correlation that produced
/// `AgentProcess::identity_status` in the first place — treat every field
/// as a claim, per the ecosystem's own doctrine that an unverified report
/// is not evidence.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize)]
pub struct SelfReportedIdentity {
    pub model: Option<String>,
    pub role: Option<String>,
    pub repository_identity: Option<String>,
    pub instance: Option<String>,
    pub task: Option<String>,
    /// Deliberately not named `capabilities`. This is what the agent
    /// *claims* it can do, not a grant or a fact about what it *can*
    /// actually do — actual/effective capabilities are a future concern
    /// meant to be derived from structural boundaries (Linux user,
    /// worktree, Guix environment, role policy), never from this field.
    pub declared_capabilities: Option<Vec<String>>,
}

/// provenance: OS-observed, as a single block rather than flat fields on
/// `AgentProcess` — so "no OS process exists for this PID right now" can
/// be represented as `None` instead of inventing empty/placeholder
/// command/cwd/start-time facts for a process that isn't there. This is
/// what makes the orphaned-identity-file-for-a-dead-PID case (see
/// `IdentityStatus::Orphaned`) representable without faking anything.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct OsObservedFacts {
    pub command: String,
    pub cwd: Option<String>,
    pub started_at_observed: Option<String>,
    /// Name of the repository (from this same scan's `repositories`)
    /// whose path contains this process's `cwd`, if any.
    pub repo_association: Option<String>,
}

/// One process relevant to the ecosystem — either a live OS process whose
/// cwd falls inside a repository from this same scan or which has a
/// self-reported identity record, or a self-reported identity record
/// whose claimed PID no longer corresponds to any live process at all
/// (`os_observed: None`, `identity_status: Orphaned`). Irrelevant OS
/// processes (the vast majority on any real machine) are never included
/// here at all; this is not a full process list.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AgentProcess {
    pub pid: u32,
    /// `None` means exactly one thing: no OS process with this `pid`
    /// exists right now. Never populated with guessed/default values.
    pub os_observed: Option<OsObservedFacts>,

    // provenance: self-reported, gated by identity_status.
    pub identity_status: IdentityStatus,
    /// All fields `None` when `identity_status` is `Orphaned` or
    /// `NotFound` — there is nothing trustworthy to report in either case.
    pub identity: SelfReportedIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EcosystemSnapshot {
    pub scan: ScanMetadata,
    pub repositories: Vec<RepositorySnapshot>,
    pub local_processes: Vec<AgentProcess>,
}
