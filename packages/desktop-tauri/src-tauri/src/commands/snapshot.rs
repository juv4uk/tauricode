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
/// — see `parse_repos()` below, added after a real portability gap was
/// found (2026-08-25): this list was a `const` (recompile required to
/// point at a different repo set), while the root path was already an
/// overridable env var. `ecosystem-observer` itself was always
/// repo-agnostic (`root`/`repositories` are `DiscoverInput` fields,
/// never baked into the crate) — this was a gap in this package's own
/// command layer, not in the observer core.
const DEFAULT_ECOSYSTEM_REPOS: [&str; 6] = [
    "my-lisp",
    "fpga-lisp",
    "cml",
    "my-idea",
    "my-lisp-panini",
    "shiva-sutras",
];

/// Pure: resolves the root path from an already-read env value (or
/// `None`). Falls back to `/home/agents/GitHub`, the path every
/// agent-facing doc in this ecosystem already assumes as the standard
/// checkout location (this session's own root `CLAUDE.md`: "every agent
/// working anywhere in `/home/agents/GitHub/*`"). Not a guess — an
/// already-established ecosystem-wide convention, made overridable for
/// any machine where it doesn't hold (e.g. CI, which does not check out
/// the five sibling repos at all — each repo scan fails independently in
/// that case, per discover_ecosystem's own per-repo-independence design,
/// not a crash). Split from env-reading so tests can exercise the logic
/// without mutating real process env vars (which race across parallel
/// `cargo test` threads).
fn resolve_root(raw: Option<&str>) -> PathBuf {
    raw.map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/home/agents/GitHub"))
}

/// Pure: parses a comma-separated repo list from an already-read env
/// value (or `None`). Falls back to this ecosystem's own six repos.
/// Empty entries from stray commas/whitespace are dropped rather than
/// passed through as a bogus repo name. Same split-for-testability
/// reasoning as `resolve_root`.
fn parse_repos(raw: Option<&str>) -> Vec<String> {
    match raw {
        Some(raw) => {
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
        None => DEFAULT_ECOSYSTEM_REPOS.iter().map(|s| s.to_string()).collect(),
    }
}

/// `ECOSYSTEM_ROOT` env var overrides the parent directory containing all
/// sibling repos.
fn ecosystem_root() -> PathBuf {
    resolve_root(std::env::var("ECOSYSTEM_ROOT").ok().as_deref())
}

/// `ECOSYSTEM_REPOS` env var (comma-separated, e.g.
/// `ECOSYSTEM_REPOS=repo-a,repo-b,repo-c`) overrides which subdirectories
/// of `ecosystem_root()` get scanned.
fn ecosystem_repos() -> Vec<String> {
    parse_repos(std::env::var("ECOSYSTEM_REPOS").ok().as_deref())
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

#[cfg(test)]
mod tests {
    use super::*;
    use ecosystem_observer::ScanStatus;
    use std::process::Command;

    #[test]
    fn parse_repos_falls_back_to_default_when_unset() {
        assert_eq!(parse_repos(None).len(), DEFAULT_ECOSYSTEM_REPOS.len());
    }

    #[test]
    fn parse_repos_splits_and_trims_custom_csv() {
        let got = parse_repos(Some("repo-a, repo-b ,repo-c"));
        assert_eq!(got, vec!["repo-a", "repo-b", "repo-c"]);
    }

    #[test]
    fn parse_repos_empty_string_falls_back_to_default() {
        assert_eq!(parse_repos(Some("")).len(), DEFAULT_ECOSYSTEM_REPOS.len());
    }

    #[test]
    fn parse_repos_drops_stray_commas() {
        let got = parse_repos(Some("repo-a,,repo-b,"));
        assert_eq!(got, vec!["repo-a", "repo-b"]);
    }

    #[test]
    fn resolve_root_falls_back_to_ecosystem_default_when_unset() {
        assert_eq!(resolve_root(None), PathBuf::from("/home/agents/GitHub"));
    }

    #[test]
    fn resolve_root_uses_override_when_given() {
        assert_eq!(resolve_root(Some("/tmp/x")), PathBuf::from("/tmp/x"));
    }

    fn git(dir: &std::path::Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .status()
            .expect("failed to spawn git in test fixture");
        assert!(status.success(), "git {args:?} failed in {dir:?}");
    }

    /// Fresh-directory portability experiment (2026-08-25), made into a
    /// real, automated, CI-reproducible test rather than only a
    /// once-off manual run. Proves the observer + this command layer's
    /// config surface is genuinely repo-set-agnostic: three repos never
    /// seen by this ecosystem (clean/main, dirty/"trunk" — a
    /// deliberately non-standard branch name, and not-a-git-repo-at-all)
    /// are scanned correctly with zero source changes, exercising the
    /// exact same `resolve_root`/`parse_repos`/`discover_ecosystem` path
    /// `get_ecosystem_snapshot` uses. Does not touch real process env
    /// vars (would race other tests running in parallel) — calls
    /// `discover_ecosystem` directly with the already-resolved
    /// `DiscoverInput`, which is the same thing `ecosystem_root()`/
    /// `ecosystem_repos()` would produce from those env vars.
    ///
    /// This test proves the NOW+NEXT portability claims only
    /// (`packages/desktop-tauri/README.md`'s own three-level table) — it
    /// does not touch contracts/tasks/evidence or the UI, which remain
    /// unproven (LATER).
    #[test]
    fn fresh_directory_portability_experiment() {
        let root = std::env::temp_dir().join(format!(
            "desktop-tauri-fresh-experiment-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();

        // repo-a: clean, on "main".
        let repo_a = root.join("repo-a");
        std::fs::create_dir_all(&repo_a).unwrap();
        git(&repo_a, &["init", "-q", "-b", "main"]);
        git(&repo_a, &["config", "user.email", "fresh@example.invalid"]);
        git(&repo_a, &["config", "user.name", "Fresh Experiment"]);
        std::fs::write(repo_a.join("README.md"), "repo-a\n").unwrap();
        git(&repo_a, &["add", "-A"]);
        git(&repo_a, &["commit", "-q", "-m", "initial commit"]);

        // repo-b: dirty, on "trunk" — deliberately not main/master, to
        // rule out any hidden default-branch assumption.
        let repo_b = root.join("repo-b");
        std::fs::create_dir_all(&repo_b).unwrap();
        git(&repo_b, &["init", "-q", "-b", "trunk"]);
        git(&repo_b, &["config", "user.email", "fresh@example.invalid"]);
        git(&repo_b, &["config", "user.name", "Fresh Experiment"]);
        std::fs::write(repo_b.join("README.md"), "repo-b\n").unwrap();
        git(&repo_b, &["add", "-A"]);
        git(&repo_b, &["commit", "-q", "-m", "initial commit"]);
        std::fs::write(repo_b.join("README.md"), "repo-b\ndirty change\n").unwrap();

        // repo-c: not a git repository at all.
        let repo_c = root.join("repo-c");
        std::fs::create_dir_all(&repo_c).unwrap();
        std::fs::write(repo_c.join("notes.txt"), "not a git repo\n").unwrap();

        // Same resolution logic get_ecosystem_snapshot uses, given what
        // ECOSYSTEM_ROOT="<root>" / ECOSYSTEM_REPOS="repo-a,repo-b,repo-c"
        // would have produced.
        let input = DiscoverInput {
            root: resolve_root(Some(root.to_str().unwrap())),
            repositories: Some(parse_repos(Some("repo-a,repo-b,repo-c"))),
            identity_base_dir: None,
        };
        let snap = discover_ecosystem(input);

        let a = snap.repositories.iter().find(|r| r.name == "repo-a").unwrap();
        assert_eq!(a.scan_status, ScanStatus::Complete);
        let a_git = a.git.as_ref().unwrap();
        assert_eq!(a_git.branch.as_deref(), Some("main"));
        assert!(!a_git.is_dirty);

        let b = snap.repositories.iter().find(|r| r.name == "repo-b").unwrap();
        assert_eq!(b.scan_status, ScanStatus::Complete);
        let b_git = b.git.as_ref().unwrap();
        assert_eq!(b_git.branch.as_deref(), Some("trunk"));
        assert!(b_git.is_dirty);
        assert_eq!(b_git.changed_paths, vec!["README.md".to_string()]);

        let c = snap.repositories.iter().find(|r| r.name == "repo-c").unwrap();
        assert_eq!(c.scan_status, ScanStatus::Failed);
        assert!(c.git.is_none());
        assert!(c.error.is_some());

        std::fs::remove_dir_all(&root).ok();
    }
}
