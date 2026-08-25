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
/// its six siblings. Only the *fallback*: this crate's own Cargo.toml
/// stays hardcoded to this ecosystem, but the command layer must not be
/// — see `ecosystem_repos()` below, added after a real portability gap
/// was found (2026-08-25): this list was a `const` (recompile required
/// to point at a different repo set), while `ecosystem_root()` was
/// already an overridable env var. `ecosystem-observer` itself was
/// always repo-agnostic (`root`/`repositories` are `DiscoverInput`
/// fields, never baked into the crate) — this was a gap in this
/// package's own command layer, not in the observer core.
const DEFAULT_ECOSYSTEM_REPOS: [&str; 6] = [
    "my-lisp",
    "fpga-lisp",
    "cml",
    "my-idea",
    "my-lisp-panini",
    "shiva-sutras",
];

/// `ECOSYSTEM_ROOT` env var overrides the parent directory containing all
/// sibling repos; falls back to `/home/agents/GitHub`, the path every
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

/// `ECOSYSTEM_REPOS` env var (comma-separated, e.g.
/// `ECOSYSTEM_REPOS=repo-a,repo-b,repo-c`) overrides which subdirectories
/// of `ecosystem_root()` get scanned; falls back to this ecosystem's own
/// six repos. Empty entries from stray commas/whitespace are dropped
/// rather than passed through as a bogus repo name.
fn ecosystem_repos() -> Vec<String> {
    match std::env::var("ECOSYSTEM_REPOS") {
        Ok(raw) => {
            let repos: Vec<String> = raw
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
            if repos.is_empty() {
                DEFAULT_ECOSYSTEM_REPOS.iter().map(|s| s.to_string()).collect()
            } else {
                repos
            }
        }
        Err(_) => DEFAULT_ECOSYSTEM_REPOS.iter().map(|s| s.to_string()).collect(),
    }
}

/// IPC Command: one real, live `EcosystemSnapshot` scan of the
/// configured sibling repos. Read-only — matches Stage 1's own explicit
/// non-goals (no launch, no git mutation, no claim/release, no Guix
/// mutation, no contract/evidence edit).
#[tauri::command]
pub async fn get_ecosystem_snapshot() -> Result<EcosystemSnapshot, String> {
    let input = DiscoverInput {
        root: ecosystem_root(),
        repositories: Some(ecosystem_repos()),
        identity_base_dir: None,
    };
    Ok(discover_ecosystem(input))
}
