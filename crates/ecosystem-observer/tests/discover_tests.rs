//! Slice 1 acceptance scenarios, per the coordinator's request:
//! clean, dirty, detached HEAD, no remote, multiple remotes, rename,
//! non-git directory, partial scan (one repo fails, rest usable), and a
//! genuine `ScanStatus::Partial` case (git repo, one probe fails, the
//! rest succeed). All fixtures are real `git init`-ed temp repositories —
//! no mocks, no PATH/env overrides (see `partial_scan_when_one_probe_fails`
//! for why that matters).
//!
//! Copy handling is deliberately NOT tested here — `git status` doesn't
//! detect copies without `--find-copies`, which this crate doesn't pass;
//! see `git_read::read_dirty_state`'s doc comment.

mod common;

use ecosystem_observer::{discover_ecosystem, scan_repository, DiscoverInput, ScanStatus};

#[test]
fn clean_repository_reports_not_dirty() {
    let repo = common::TempDir::new("clean");
    common::init_repo(&repo.path);
    std::fs::write(repo.path.join("file.txt"), "hello\n").unwrap();
    common::commit_all(&repo.path, "init");

    let snap = scan_repository("clean", &repo.path);

    assert_eq!(snap.scan_status, ScanStatus::Complete);
    let git = snap.git.expect("git state present");
    assert_eq!(git.branch.as_deref(), Some("main"));
    assert!(!git.is_detached);
    assert!(git.head_sha.is_some());
    assert!(!git.is_dirty);
    assert!(git.changed_paths.is_empty());
    assert!(git.remotes.is_empty());
}

#[test]
fn dirty_repository_lists_changed_paths() {
    let repo = common::TempDir::new("dirty");
    common::init_repo(&repo.path);
    std::fs::write(repo.path.join("file.txt"), "hello\n").unwrap();
    common::commit_all(&repo.path, "init");
    std::fs::write(repo.path.join("file.txt"), "hello again\n").unwrap();
    std::fs::write(repo.path.join("untracked.txt"), "new\n").unwrap();

    let snap = scan_repository("dirty", &repo.path);

    assert_eq!(snap.scan_status, ScanStatus::Complete);
    let git = snap.git.expect("git state present");
    assert!(git.is_dirty);
    assert_eq!(git.changed_paths.len(), 2);
    assert!(git.changed_paths.iter().any(|p| p == "file.txt"));
    assert!(git.changed_paths.iter().any(|p| p == "untracked.txt"));
}

#[test]
fn detached_head_reports_no_branch_but_a_sha() {
    let repo = common::TempDir::new("detached");
    common::init_repo(&repo.path);
    std::fs::write(repo.path.join("a.txt"), "a\n").unwrap();
    common::commit_all(&repo.path, "init");
    let sha = common::head_sha(&repo.path);
    common::git(&repo.path, &["checkout", "-q", "--detach", &sha]);

    let snap = scan_repository("detached", &repo.path);

    assert_eq!(snap.scan_status, ScanStatus::Complete);
    let git = snap.git.expect("git state present");
    assert!(git.is_detached);
    assert_eq!(git.branch, None);
    assert_eq!(git.head_sha.as_deref(), Some(sha.as_str()));
}

#[test]
fn missing_origin_reports_empty_remotes_not_an_error() {
    let repo = common::TempDir::new("noremote");
    common::init_repo(&repo.path);
    std::fs::write(repo.path.join("a.txt"), "a\n").unwrap();
    common::commit_all(&repo.path, "init");

    let snap = scan_repository("noremote", &repo.path);

    assert_eq!(snap.scan_status, ScanStatus::Complete);
    assert!(snap.git.expect("git state present").remotes.is_empty());
}

#[test]
fn multiple_remotes_are_all_observed() {
    let repo = common::TempDir::new("multiremote");
    common::init_repo(&repo.path);
    std::fs::write(repo.path.join("a.txt"), "a\n").unwrap();
    common::commit_all(&repo.path, "init");
    common::git(
        &repo.path,
        &[
            "remote",
            "add",
            "origin",
            "https://example.invalid/origin.git",
        ],
    );
    common::git(
        &repo.path,
        &[
            "remote",
            "add",
            "upstream",
            "https://example.invalid/upstream.git",
        ],
    );

    let snap = scan_repository("multiremote", &repo.path);

    let git = snap.git.expect("git state present");
    assert_eq!(git.remotes.len(), 2);
    assert!(git
        .remotes
        .iter()
        .any(|r| r.name == "origin" && r.url == "https://example.invalid/origin.git"));
    assert!(git
        .remotes
        .iter()
        .any(|r| r.name == "upstream" && r.url == "https://example.invalid/upstream.git"));
}

/// Field order for the `-z` porcelain rename format (new path, then old
/// path) was verified empirically against git 2.43.0 before writing
/// `git_read::read_dirty_state` — this test locks that behavior in.
#[test]
fn rename_and_modify_is_reported_as_old_arrow_new() {
    let repo = common::TempDir::new("rename");
    common::init_repo(&repo.path);
    std::fs::write(repo.path.join("old.txt"), "hello world\n").unwrap();
    common::commit_all(&repo.path, "init");
    common::git(&repo.path, &["mv", "old.txt", "new.txt"]);
    std::fs::write(repo.path.join("new.txt"), "hello world\nextra line\n").unwrap();

    let snap = scan_repository("rename", &repo.path);

    let git = snap.git.expect("git state present");
    assert!(git.is_dirty);
    assert_eq!(git.changed_paths, vec!["old.txt -> new.txt".to_string()]);
}

#[test]
fn pure_rename_without_content_change_is_also_reported() {
    let repo = common::TempDir::new("purerename");
    common::init_repo(&repo.path);
    std::fs::write(repo.path.join("old2.txt"), "unchanged content\n").unwrap();
    common::commit_all(&repo.path, "init");
    common::git(&repo.path, &["mv", "old2.txt", "new2.txt"]);

    let snap = scan_repository("purerename", &repo.path);

    let git = snap.git.expect("git state present");
    assert!(git.is_dirty);
    assert_eq!(git.changed_paths, vec!["old2.txt -> new2.txt".to_string()]);
}

#[test]
fn non_git_directory_is_reported_failed_not_panicking() {
    let dir = common::TempDir::new("nongit");
    // deliberately no `git init`, and not nested inside any other repo
    // (std::env::temp_dir() is not itself a git working tree) — this is
    // the "genuinely no repository anywhere in reach" case, distinct from
    // `nested_non_git_directory_inside_repo_does_not_inherit_parent_state`
    // below, which is "no repository *at this exact path*, but one exists
    // above it".

    let snap = scan_repository("nongit", &dir.path);

    assert_eq!(snap.scan_status, ScanStatus::Failed);
    assert!(snap.git.is_none());
    assert_eq!(
        snap.error.as_deref(),
        Some("no git repository found at this path or in any ancestor directory")
    );
}

#[test]
fn nonexistent_path_is_reported_failed_not_panicking() {
    let root = common::TempDir::new("nonexistent-parent");
    let missing = root.path.join("does-not-exist");

    let snap = scan_repository("missing", &missing);

    assert_eq!(snap.scan_status, ScanStatus::Failed);
    assert!(snap.git.is_none());
    assert_eq!(snap.error.as_deref(), Some("path does not exist"));
}

/// A real `ScanStatus::Partial`: the repository is valid (`is_git_repo`
/// gate passes, branch/head-sha/remotes all read successfully), but the
/// dirty-state probe fails because `.git/index` is corrupted. Verified
/// empirically before writing this test (see the coordinator's session
/// notes): corrupting the index breaks exactly `git status`, while
/// `rev-parse --is-inside-work-tree`, `symbolic-ref --short HEAD`,
/// `rev-parse HEAD`, and `remote -v` all keep succeeding — a precise,
/// filesystem-only fault injection with no global PATH/env mutation
/// (which would risk interfering with other tests running in parallel
/// in the same process).
#[test]
fn partial_scan_when_one_probe_fails() {
    let repo = common::TempDir::new("partial");
    common::init_repo(&repo.path);
    std::fs::write(repo.path.join("a.txt"), "a\n").unwrap();
    common::commit_all(&repo.path, "init");
    common::git(
        &repo.path,
        &[
            "remote",
            "add",
            "origin",
            "https://example.invalid/origin.git",
        ],
    );
    let sha = common::head_sha(&repo.path);

    // Corrupt the index so `git status --porcelain=v1 -z` fails while the
    // other three probes remain unaffected.
    std::fs::write(repo.path.join(".git").join("index"), [0u8; 200]).unwrap();

    let snap = scan_repository("partial", &repo.path);

    assert_eq!(snap.scan_status, ScanStatus::Partial);
    assert!(
        snap.error.is_some(),
        "error summary should be populated for Partial too"
    );
    let git = snap.git.expect("git state present even though partial");

    // facts that WERE read successfully must be preserved, not discarded:
    assert_eq!(git.branch.as_deref(), Some("main"));
    assert!(!git.is_detached);
    assert_eq!(git.head_sha.as_deref(), Some(sha.as_str()));
    assert_eq!(git.remotes.len(), 1);
    assert_eq!(git.remotes[0].name, "origin");

    // the one probe that failed is named, not silently absorbed into a
    // default `false`/`empty` that would look identical to "clean"/"no changes":
    assert_eq!(git.unavailable.len(), 1);
    assert_eq!(git.unavailable[0].probe, "dirty");
    assert!(!git.unavailable[0].reason.is_empty());

    // the placeholder values for the failed probe are exactly that —
    // placeholders, not to be read as "clean":
    assert!(!git.is_dirty);
    assert!(git.changed_paths.is_empty());
}

/// Snapshot-level "partial coverage" (some repos `Complete`, some
/// `Failed`, snapshot as a whole still usable) — distinct from the
/// per-repository `ScanStatus::Partial` exercised by
/// `partial_scan_when_one_probe_fails` above. Named without "partial" to
/// avoid conflating the two.
#[test]
fn one_repository_failing_does_not_affect_the_others() {
    let root = common::TempDir::new("ecosystem-root");
    for name in ["repo-a", "repo-b"] {
        let path = root.path.join(name);
        std::fs::create_dir_all(&path).unwrap();
        common::init_repo(&path);
        std::fs::write(path.join("f.txt"), "x\n").unwrap();
        common::commit_all(&path, "init");
    }
    // repo-c: a real directory, deliberately never `git init`-ed.
    std::fs::create_dir_all(root.path.join("repo-c")).unwrap();
    // repo-d: named explicitly but never created on disk at all.

    let snapshot = discover_ecosystem(DiscoverInput {
        root: root.path.clone(),
        repositories: Some(vec![
            "repo-a".to_string(),
            "repo-b".to_string(),
            "repo-c".to_string(),
            "repo-d".to_string(),
        ]),
        identity_base_dir: None,
    });

    assert_eq!(snapshot.repositories.len(), 4);
    let status_of = |name: &str| {
        snapshot
            .repositories
            .iter()
            .find(|r| r.name == name)
            .unwrap()
            .scan_status
    };
    assert_eq!(status_of("repo-a"), ScanStatus::Complete);
    assert_eq!(status_of("repo-b"), ScanStatus::Complete);
    assert_eq!(status_of("repo-c"), ScanStatus::Failed);
    assert_eq!(status_of("repo-d"), ScanStatus::Failed);

    // the snapshot as a whole is still usable and carries scan identity,
    // even though it is a mix of complete and failed repositories.
    assert!(!snapshot.scan.scan_id.is_empty());
    assert!(!snapshot.scan.started_at.is_empty());
    assert!(!snapshot.scan.finished_at.is_empty());
}

#[test]
fn auto_discovers_all_subdirectories_when_repositories_not_specified() {
    let root = common::TempDir::new("auto");
    for name in ["z-repo", "a-repo"] {
        let path = root.path.join(name);
        std::fs::create_dir_all(&path).unwrap();
        common::init_repo(&path);
        std::fs::write(path.join("f.txt"), "x\n").unwrap();
        common::commit_all(&path, "init");
    }

    let snapshot = discover_ecosystem(DiscoverInput {
        root: root.path.clone(),
        repositories: None,
        identity_base_dir: None,
    });

    let mut names: Vec<&str> = snapshot
        .repositories
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    names.sort();
    assert_eq!(names, vec!["a-repo", "z-repo"]);
}

// --- Hardening regressions (repository-identity fix, bare repos, symlinks, timeout) ---

/// The bug this locks in: a plain, non-git subdirectory nested inside a
/// git working tree used to be reported `ScanStatus::Complete` with the
/// *parent* repository's branch/HEAD/remotes, because
/// `rev-parse --is-inside-work-tree` succeeds via git's own upward
/// directory search. `identify_repository`'s `--show-toplevel`-vs-`path`
/// identity check closes this.
#[test]
fn nested_non_git_directory_inside_repo_does_not_inherit_parent_state() {
    let parent = common::TempDir::new("nested-parent");
    common::init_repo(&parent.path);
    std::fs::write(parent.path.join("a.txt"), "a\n").unwrap();
    common::commit_all(&parent.path, "init");
    common::git(
        &parent.path,
        &[
            "remote",
            "add",
            "origin",
            "https://example.invalid/parent.git",
        ],
    );
    let nested = parent.path.join("plain-subdir");
    std::fs::create_dir_all(&nested).unwrap();

    let snap = scan_repository("plain-subdir", &nested);

    assert_eq!(snap.scan_status, ScanStatus::Failed);
    assert!(
        snap.git.is_none(),
        "must not carry the parent's GitState under the child's name"
    );
    let error = snap.error.expect("explicit error, not silent success");
    assert!(
        error.contains("not that repository's own root")
            || error.contains("uninitialized submodule"),
        "error should explain this is inside-but-not-root, got: {error}"
    );
}

/// Same bug, real-world shape: an uninitialized git submodule is exactly
/// an empty directory nested inside a parent working tree.
#[test]
fn uninitialized_submodule_does_not_inherit_parent_state() {
    let target = common::TempDir::new("submodule-target");
    common::init_repo(&target.path);
    std::fs::write(target.path.join("s.txt"), "s\n").unwrap();
    common::commit_all(&target.path, "init");

    let superproject = common::TempDir::new("superproject");
    common::init_repo(&superproject.path);
    common::submodule_add(&superproject.path, target.path.to_str().unwrap(), "sub");
    common::commit_all(&superproject.path, "add submodule");

    let clone_root = common::TempDir::new("superproject-clone-root");
    let clone_path = clone_root.path.join("clone");
    common::clone_repo(&superproject.path, &clone_path);
    // deliberately no `git submodule update --init` — `clone_path/sub` is
    // now an empty directory, the uninitialized-submodule shape.

    let sub_path = clone_path.join("sub");
    assert!(
        sub_path.is_dir(),
        "submodule dir should exist, just empty/uninitialized"
    );

    let snap = scan_repository("sub", &sub_path);

    assert_eq!(snap.scan_status, ScanStatus::Failed);
    assert!(snap.git.is_none());
    // must NOT be the superproject's own head/branch/remote:
    let superproject_snap = scan_repository("superproject", &clone_path);
    assert_eq!(superproject_snap.scan_status, ScanStatus::Complete);
    assert_ne!(
        snap.error, superproject_snap.error,
        "sanity: these two calls must genuinely differ, not coincidentally match"
    );
}

/// The identity check must still accept a *linked worktree* — its own
/// `--show-toplevel` equals its own path, even though its git-dir lives
/// under the main repository's `.git/worktrees/...`.
#[test]
fn linked_worktree_is_still_accepted_as_complete() {
    let main_repo = common::TempDir::new("worktree-main");
    common::init_repo(&main_repo.path);
    std::fs::write(main_repo.path.join("a.txt"), "a\n").unwrap();
    common::commit_all(&main_repo.path, "init");
    common::git(&main_repo.path, &["branch", "feature"]);

    let worktree_root = common::TempDir::new("worktree-copy-root");
    let worktree_path = worktree_root.path.join("copy");
    common::worktree_add(&main_repo.path, &worktree_path, "feature");

    let snap = scan_repository("worktree-copy", &worktree_path);

    assert_eq!(snap.scan_status, ScanStatus::Complete);
    let git = snap.git.expect("git state present");
    assert_eq!(git.branch.as_deref(), Some("feature"));
    assert!(!git.is_detached);
}

/// A bare repository IS a git repository — the error must say so
/// truthfully, never "not a git repository".
#[test]
fn bare_repository_is_classified_truthfully_not_as_missing() {
    let dir = common::TempDir::new("bare");
    common::init_bare_repo(&dir.path);

    let snap = scan_repository("bare", &dir.path);

    assert_eq!(snap.scan_status, ScanStatus::Failed);
    assert!(snap.git.is_none());
    let error = snap.error.expect("explicit error");
    assert!(
        error.contains("bare repository"),
        "error should name it as bare, got: {error}"
    );
    assert!(
        !error.contains("not a git repository"),
        "must not claim a bare repo isn't a repo at all, got: {error}"
    );
}

/// A broken symlink in the auto-discovered root must not silently vanish
/// from the snapshot.
#[test]
fn broken_symlink_in_auto_discovery_is_visible_not_hidden() {
    let root = common::TempDir::new("symlink-root");
    let real_repo = root.path.join("real-repo");
    std::fs::create_dir_all(&real_repo).unwrap();
    common::init_repo(&real_repo);
    std::fs::write(real_repo.join("a.txt"), "a\n").unwrap();
    common::commit_all(&real_repo, "init");

    #[cfg(unix)]
    std::os::unix::fs::symlink(
        root.path.join("does-not-exist"),
        root.path.join("broken-link"),
    )
    .unwrap();

    let snapshot = discover_ecosystem(DiscoverInput {
        root: root.path.clone(),
        repositories: None,
        identity_base_dir: None,
    });

    let names: Vec<&str> = snapshot
        .repositories
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    assert!(
        names.contains(&"broken-link"),
        "broken symlink must appear in the snapshot, got entries: {names:?}"
    );
    let link_entry = snapshot
        .repositories
        .iter()
        .find(|r| r.name == "broken-link")
        .unwrap();
    assert_eq!(link_entry.scan_status, ScanStatus::Failed);
    assert!(link_entry.error.is_some());
}

/// The real end-to-end timeout regression: a repository whose
/// `.git/config` hangs (via `include.path` pointing at an unread FIFO —
/// see `common::make_repo_with_hanging_config`) must produce `Failed`
/// with an error mentioning the timeout, not block forever. This test
/// genuinely takes ~20s (the production timeout) — that's the honest
/// price of testing this specific property for real rather than mocking
/// it away; see `git_read::tests::run_with_timeout_kills_a_hung_process_instead_of_blocking`
/// for a fast (sub-second) proof of the underlying mechanism using a
/// generic `sleep` instead of a real git hang, if a quick sanity check is
/// what's needed instead.
#[test]
fn timed_out_git_probe_gives_failed_not_a_hang() {
    let repo = common::TempDir::new("hanging-config");
    common::make_repo_with_hanging_config(&repo.path);

    let started = std::time::Instant::now();
    let snap = scan_repository("hanging-config", &repo.path);
    let elapsed = started.elapsed();

    assert_eq!(snap.scan_status, ScanStatus::Failed);
    let error = snap.error.expect("explicit error, not a hang");
    assert!(
        error.contains("timed out"),
        "error should mention the timeout, got: {error}"
    );
    // generous upper bound: the 20s production timeout plus real slack,
    // proving this returned because of the timeout firing, not because
    // it happened to hang forever and the test harness itself gave up.
    assert!(
        elapsed < std::time::Duration::from_secs(30),
        "took {elapsed:?}, expected to return shortly after the ~20s timeout"
    );
}
