//! `ecosystem-observer` — observer core for the my-lisp ecosystem.
//!
//! Slice 1 (repository discovery + read-only git scanning), per
//! `ECO-DECISION-2026-08-19-TAURICODE-STAGE1-OBSERVER`, placed here by
//! `ECO-DECISION-2026-08-19-TAURICODE-TAURI-ARCHITECTURE`: `GitState`,
//! read-only git scanning (`rev-parse`, `symbolic-ref`, `status`,
//! `remote -v` only).
//!
//! Slice 2 ("Local Runtime Observation + Agent Identity Contract"):
//! `AgentProcess`, two structurally separate provenance layers —
//! OS-observed (`/proc`, read-only, no signals) and self-reported (the
//! Agent Identity Contract v1, arbitrary JSON this crate only *reads*,
//! never writes; see `identity_contract`'s module doc for the format).
//! `IdentityStatus` (`Fresh`/`Stale`/`Orphaned`/`NotFound`) is a
//! deliberately separate axis from provenance, not folded into it.
//!
//! Still out of scope: Guix evaluation, tasks/evidence/contract parsing,
//! Tauri commands, frontend, OpenCode sidecar, agent-runtime control, an
//! S-expression reader (neither slice has needed to parse a `.my` file
//! yet).
//!
//! Every operation this crate performs is read-only. Nothing here writes
//! to a repository, claims a task, signals a process, or launches
//! anything other than `git`/`getconf` (both invoked read-only, for
//! plumbing queries only).

mod discover;
mod git_read;
mod identity_contract;
mod process_observe;
mod snapshot;
mod time_util;

pub use discover::{discover_ecosystem, scan_repository, DiscoverInput};
pub use process_observe::{read_start_token, OsProcess};
pub use snapshot::{
    AgentProcess, EcosystemSnapshot, GitState, IdentityStatus, OsObservedFacts, ProbeFailure,
    RemoteInfo, RepositorySnapshot, ScanMetadata, ScanStatus, SelfReportedIdentity,
};
pub use time_util::{iso8601_from_unix_seconds, iso8601_now};
