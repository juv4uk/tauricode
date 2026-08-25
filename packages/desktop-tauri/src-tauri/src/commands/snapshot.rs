// ============================================================================
// Tauricode official agent-runtime-ui — Slice 1: EcosystemSnapshot command
// Location: src-tauri/src/commands/snapshot.rs
// ============================================================================
//
// Per ECO-DECISION-2026-08-19-TAURICODE-STAGE1-OBSERVER's own architectural
// boundary: "observer core forms EcosystemSnapshot, GUI only displays it" —
// this command does no git/file reading of its own, it calls
// `ecosystem_observer::discover_ecosystem` (the real Slice 1+2 core, real
// git plumbing, real /proc reads) and returns exactly what it produces.
// Never mock/fixture data - the whole point of this package, as distinct
// from prototype/swarm_dashboard/, is that this is real observation.

use ecosystem_observer::{discover_ecosystem, DiscoverInput, EcosystemSnapshot};
use std::path::PathBuf;

/// Stage 1's declared observation scope
/// (ECO-DECISION-2026-08-19-TAURICODE-STAGE1-OBSERVER: "Scope of
/// observation") - tauricode does not observe itself here, it observes
/// its six siblings.
const ECOSYSTEM_REPOS: [&str; 6] = [
    "my-lisp",
    "fpga-lisp",
    "cml",
    "my-idea",
    "my-lisp-panini",
    "shiva-sutras",
];

/// `ECOSYSTEM_ROOT` env var overrides the parent directory containing all
/// six sibling repos; falls back to `/home/agents/GitHub`, the path every
/// agent-facing doc in this ecosystem already assumes as the standard
/// checkout location (this session's own root `CLAUDE.md`: "every agent
/// working anywhere in `/home/agents/GitHub/*`"). Not a guess — an
/// already-established ecosystem-wide convention, made overridable for
/// any machine where it doesn't hold (e.g. CI, which does not check out
/// the five sibling repos at all — each repo scan fails independently in
/// that case, per discover_ecosystem's own per-repo-independence design,
/// not a crash).
fn ecosystem_root() -> PathBuf {
    std::env::var("ECOSYSTEM_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/agents/GitHub"))
}

/// IPC Command: one real, live `EcosystemSnapshot` scan of the six
/// ecosystem sibling repos. Read-only — matches Stage 1's own explicit
/// non-goals (no launch, no git mutation, no claim/release, no Guix
/// mutation, no contract/evidence edit).
#[tauri::command]
pub async fn get_ecosystem_snapshot() -> Result<EcosystemSnapshot, String> {
    let input = DiscoverInput {
        root: ecosystem_root(),
        repositories: Some(ECOSYSTEM_REPOS.iter().map(|s| s.to_string()).collect()),
        identity_base_dir: None,
    };
    Ok(discover_ecosystem(input))
}
