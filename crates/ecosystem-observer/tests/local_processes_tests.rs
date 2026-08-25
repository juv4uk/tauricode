//! Slice 2 end-to-end regressions: real spawned OS processes plus real
//! identity JSON files on disk, read through the public
//! `discover_ecosystem` entry point — not unit-level mocks of the
//! correlation math (those already live in `identity_contract`'s own
//! `#[cfg(test)]` module). `identity_base_dir` is always overridden to an
//! isolated temp dir per test, never the real `$XDG_RUNTIME_DIR`, so
//! these never race against each other or against a real agent's files.

// This binary only uses `common::TempDir` - the git-fixture helpers
// (`git`/`init_repo`/`commit_all`/etc.) are exercised by
// `discover_tests.rs`'s own compilation of this same shared module.
// Each `tests/*.rs` file compiles `tests/common/mod.rs` as its own
// separate crate, so "unused" is genuinely per-binary here, not a sign
// of real dead code in the shared module itself.
#[allow(dead_code)]
mod common;

use ecosystem_observer::{
    discover_ecosystem, iso8601_now, read_start_token, DiscoverInput, IdentityStatus,
};
use std::process::Command;
use std::time::Duration;

/// A PID far above any value Linux would plausibly assign right now
/// (default `pid_max` is commonly 4194304) — used for the "identity file
/// references a PID that was never real" scenario, deterministically,
/// without racing a just-exited process's PID being reused.
const IMPOSSIBLE_PID: u32 = 4_000_111;

fn write_identity_file(dir: &std::path::Path, pid: u32, json: &str) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join(format!("{pid}.json")), json).unwrap();
}

fn identity_json(
    pid: u32,
    token: Option<&str>,
    started_at: Option<&str>,
    updated_at: Option<&str>,
) -> String {
    format!(
        r#"{{"schema_version":1,"pid":{pid},"process_start_token":{token},"started_at":{started_at},"updated_at":{updated_at},"model":"Claude Sonnet 5","role":"Ecosystem Lead","repository":"tauricode","instance":null,"task":"testing","declared_capabilities":["bash"]}}"#,
        pid = pid,
        token = opt_str(token),
        started_at = opt_str(started_at),
        updated_at = opt_str(updated_at),
    )
}

fn opt_str(v: Option<&str>) -> String {
    match v {
        Some(s) => format!("{s:?}"),
        None => "null".to_string(),
    }
}

/// Spawns `sleep 5` with the given cwd, waits briefly for its `/proc`
/// entry to be readable, and returns the child plus its real
/// `process_start_token` (read the same way production code would).
fn spawn_tracked(cwd: &std::path::Path) -> (std::process::Child, String) {
    let child = Command::new("sleep")
        .arg("5")
        .current_dir(cwd)
        .spawn()
        .expect("failed to spawn test process");
    std::thread::sleep(Duration::from_millis(150));
    let token = read_start_token(child.id()).expect("should read start token for a live process");
    (child, token)
}

fn kill(mut child: std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn fresh_for_a_real_process_with_matching_token_and_recent_heartbeat() {
    let root = common::TempDir::new("lp-fresh-root");
    let repo_dir = root.path.join("repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    let identity_dir = common::TempDir::new("lp-fresh-identity");

    let (child, token) = spawn_tracked(&repo_dir);
    let json = identity_json(child.id(), Some(&token), None, Some(&iso8601_now()));
    write_identity_file(&identity_dir.path, child.id(), &json);

    let snapshot = discover_ecosystem(DiscoverInput {
        root: root.path.clone(),
        repositories: Some(vec!["repo".to_string()]),
        identity_base_dir: Some(identity_dir.path.clone()),
    });

    let process = snapshot
        .local_processes
        .iter()
        .find(|p| p.pid == child.id())
        .expect("spawned process should appear in local_processes");
    assert_eq!(process.identity_status, IdentityStatus::Fresh);
    let os = process
        .os_observed
        .as_ref()
        .expect("a live, relevant process must have os_observed: Some");
    assert_eq!(os.repo_association.as_deref(), Some("repo"));
    assert!(!os.command.is_empty());
    assert_eq!(process.identity.model.as_deref(), Some("Claude Sonnet 5"));
    assert_eq!(process.identity.role.as_deref(), Some("Ecosystem Lead"));

    kill(child);
}

#[test]
fn stale_for_a_real_process_with_matching_token_but_old_heartbeat() {
    let root = common::TempDir::new("lp-stale-root");
    let repo_dir = root.path.join("repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    let identity_dir = common::TempDir::new("lp-stale-identity");

    let (child, token) = spawn_tracked(&repo_dir);
    let ancient = ecosystem_observer::iso8601_from_unix_seconds(0); // 1970 — far past the 60s threshold
    let json = identity_json(child.id(), Some(&token), None, Some(&ancient));
    write_identity_file(&identity_dir.path, child.id(), &json);

    let snapshot = discover_ecosystem(DiscoverInput {
        root: root.path.clone(),
        repositories: Some(vec!["repo".to_string()]),
        identity_base_dir: Some(identity_dir.path.clone()),
    });

    let process = snapshot
        .local_processes
        .iter()
        .find(|p| p.pid == child.id())
        .expect("spawned process should appear in local_processes");
    assert_eq!(process.identity_status, IdentityStatus::Stale);
    assert!(process.os_observed.is_some());
    // facts already reported are preserved during Stale, same principle
    // as Slice 1's GitState::Partial.
    assert_eq!(process.identity.model.as_deref(), Some("Claude Sonnet 5"));

    kill(child);
}

#[test]
fn orphaned_when_a_real_processs_identity_file_has_a_mismatched_token() {
    let root = common::TempDir::new("lp-orphan-token-root");
    let repo_dir = root.path.join("repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    let identity_dir = common::TempDir::new("lp-orphan-token-identity");

    let (child, _real_token) = spawn_tracked(&repo_dir);
    // deliberately wrong token: simulates a stale file surviving PID reuse
    let json = identity_json(child.id(), Some("999999999"), None, Some(&iso8601_now()));
    write_identity_file(&identity_dir.path, child.id(), &json);

    let snapshot = discover_ecosystem(DiscoverInput {
        root: root.path.clone(),
        repositories: Some(vec!["repo".to_string()]),
        identity_base_dir: Some(identity_dir.path.clone()),
    });

    let process = snapshot
        .local_processes
        .iter()
        .find(|p| p.pid == child.id())
        .expect("spawned process should still appear (it's real and repo-associated)");
    assert_eq!(process.identity_status, IdentityStatus::Orphaned);
    // the process is real, so os_observed must still be populated —
    // orphaning is about distrusting the *identity claim*, not about
    // pretending the process itself doesn't exist.
    assert!(process.os_observed.is_some());
    assert_eq!(process.identity.model, None);

    kill(child);
}

#[test]
fn orphaned_when_identity_file_references_a_pid_that_was_never_real() {
    let root = common::TempDir::new("lp-orphan-missing-root");
    let identity_dir = common::TempDir::new("lp-orphan-missing-identity");

    let json = identity_json(IMPOSSIBLE_PID, Some("42"), None, Some(&iso8601_now()));
    write_identity_file(&identity_dir.path, IMPOSSIBLE_PID, &json);

    let snapshot = discover_ecosystem(DiscoverInput {
        root: root.path.clone(),
        repositories: None,
        identity_base_dir: Some(identity_dir.path.clone()),
    });

    let process = snapshot
        .local_processes
        .iter()
        .find(|p| p.pid == IMPOSSIBLE_PID)
        .expect(
            "orphaned identity file for a dead PID must still be surfaced, not silently dropped",
        );
    assert_eq!(process.identity_status, IdentityStatus::Orphaned);
    assert!(
        process.os_observed.is_none(),
        "must never invent command/cwd/start facts for a process that was never observed"
    );
    assert_eq!(process.identity.model, None);
}

#[test]
fn not_found_for_a_real_relevant_process_with_no_identity_file_at_all() {
    let root = common::TempDir::new("lp-notfound-root");
    let repo_dir = root.path.join("repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    let identity_dir = common::TempDir::new("lp-notfound-identity"); // exists but stays empty

    let child = Command::new("sleep")
        .arg("5")
        .current_dir(&repo_dir)
        .spawn()
        .expect("failed to spawn test process");
    std::thread::sleep(Duration::from_millis(150));

    let snapshot = discover_ecosystem(DiscoverInput {
        root: root.path.clone(),
        repositories: Some(vec!["repo".to_string()]),
        identity_base_dir: Some(identity_dir.path.clone()),
    });

    let process = snapshot
        .local_processes
        .iter()
        .find(|p| p.pid == child.id())
        .expect("repo-associated process should appear even without an identity file");
    assert_eq!(process.identity_status, IdentityStatus::NotFound);
    assert!(process.os_observed.is_some());
    assert_eq!(process.identity, Default::default());

    kill(child);
}

#[test]
fn fresh_via_started_at_tolerance_fallback_when_process_start_token_absent() {
    let root = common::TempDir::new("lp-fallback-root");
    let repo_dir = root.path.join("repo");
    std::fs::create_dir_all(&repo_dir).unwrap();
    let identity_dir = common::TempDir::new("lp-fallback-identity");

    let child = Command::new("sleep")
        .arg("5")
        .current_dir(&repo_dir)
        .spawn()
        .expect("failed to spawn test process");
    std::thread::sleep(Duration::from_millis(150));
    // no process_start_token at all — claimed started_at is "now", which
    // is within the 5s tolerance of the real, just-spawned process.
    let json = identity_json(child.id(), None, Some(&iso8601_now()), Some(&iso8601_now()));
    write_identity_file(&identity_dir.path, child.id(), &json);

    let snapshot = discover_ecosystem(DiscoverInput {
        root: root.path.clone(),
        repositories: Some(vec!["repo".to_string()]),
        identity_base_dir: Some(identity_dir.path.clone()),
    });

    let process = snapshot
        .local_processes
        .iter()
        .find(|p| p.pid == child.id())
        .expect("spawned process should appear in local_processes");
    assert_eq!(process.identity_status, IdentityStatus::Fresh);
    assert_eq!(process.identity.model.as_deref(), Some("Claude Sonnet 5"));

    kill(child);
}
