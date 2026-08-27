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
//! `sexpr` (2026-08-25): a minimal S-expression reader — symbols,
//! strings, integers, lists, dotted pairs, `t`/`()`-truthiness, `;`
//! comments; no eval, no macro-expansion, no my-lisp dependency, per
//! `ECO-DECISION-2026-08-19-TAURICODE-TAURI-ARCHITECTURE` point 3.
//!
//! `contracts` (2026-08-27): typed parsing on top of `sexpr` for
//! `language-contract.my`/`ecosystem-status.my`, plus
//! `detect_language_contract_drift` — the first concrete piece of Stage
//! 1's §3 "Ecosystem contracts" acceptance criteria, including its
//! mandatory drift-detection gate (see the module's own doc comment).
//! Still not done: `repo.my` typed parsing beyond the existing
//! `Expr::assoc`/`tagged_list` helpers, `isa-contract.my`,
//! `compatibility.my`, `tasks.my`/`evidence/` typed parsing, and wiring
//! any of this into `EcosystemSnapshot` — all deliberately deferred,
//! reader/parser landed unwired first, same precedent as `sexpr` itself.
//!
//! Still out of scope: Guix evaluation, Tauri commands beyond
//! `get_ecosystem_snapshot`, frontend beyond the plain `web/index.html`
//! proof, OpenCode sidecar, agent-runtime control.
//!
//! Every operation this crate performs is read-only. Nothing here writes
//! to a repository, claims a task, signals a process, or launches
//! anything other than `git`/`getconf` (both invoked read-only, for
//! plumbing queries only).

mod contracts;
mod discover;
mod git_read;
mod identity_contract;
mod process_observe;
mod sexpr;
mod snapshot;
mod time_util;

pub use contracts::{
    detect_language_contract_drift, parse_claimed_language_contract_version,
    parse_language_contract_version, ContractDrift, ContractError, ContractVersion,
};
pub use discover::{discover_ecosystem, scan_repository, DiscoverInput};
pub use process_observe::{read_start_token, OsProcess};
pub use sexpr::{parse as parse_sexpr, Expr as SexprExpr, ParseError as SexprParseError};
pub use snapshot::{
    AgentProcess, EcosystemSnapshot, GitState, IdentityStatus, OsObservedFacts, ProbeFailure,
    RemoteInfo, RepositorySnapshot, ScanMetadata, ScanStatus, SelfReportedIdentity,
};
pub use time_util::{iso8601_from_unix_seconds, iso8601_now};
