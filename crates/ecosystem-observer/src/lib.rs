//! `ecosystem-observer` — Slice 1 (repository discovery + read-only git
//! scanning) of the observer core defined by
//! `ECO-DECISION-2026-08-19-TAURICODE-STAGE1-OBSERVER` and placed in
//! tauricode by `ECO-DECISION-2026-08-19-TAURICODE-TAURI-ARCHITECTURE`.
//!
//! Scope of this slice, deliberately: `EcosystemSnapshot`/
//! `RepositorySnapshot`/`GitState`/`ScanMetadata` and read-only git
//! scanning only. No Guix evaluation, no tasks/evidence/contract parsing,
//! no Tauri commands, no frontend, no OpenCode sidecar, no agent-runtime
//! control, no S-expression reader (this slice never needed to parse an
//! S-expression file — see the coordinator's slice report for why one
//! wasn't added).
//!
//! Every operation this crate performs is read-only: it shells out to
//! `git` for plumbing queries only (`rev-parse`, `symbolic-ref`, `status`,
//! `remote -v`) and to `std::fs::read_dir` for directory listing. Nothing
//! here writes to a repository, claims a task, or launches a process other
//! than `git` itself.

mod discover;
mod git_read;
mod snapshot;
mod time_util;

pub use discover::{discover_ecosystem, scan_repository, DiscoverInput};
pub use snapshot::{
    EcosystemSnapshot, GitState, ProbeFailure, RemoteInfo, RepositorySnapshot, ScanMetadata,
    ScanStatus,
};
