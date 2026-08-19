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
    // deliberately no `git init`

    let snap = scan_repository("nongit", &dir.path);

    assert_eq!(snap.scan_status, ScanStatus::Failed);
    assert!(snap.git.is_none());
    assert_eq!(snap.error.as_deref(), Some("not a git repository"));
}

#[test]
fn nonexistent_path_is_reported_failed_not_panicking() {
    let root = common::TempDir::new("nonexistent-parent");
    let missing = root.path.join("does-not-exist");

    let snap = scan_repository("missing", &missing);

    assert_eq!(snap.scan_status, ScanStatus::Failed);
    assert!(snap.git.is_none());
    assert!(snap.error.is_some());
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
    });

    let mut names: Vec<&str> = snapshot
        .repositories
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    names.sort();
    assert_eq!(names, vec!["a-repo", "z-repo"]);
}
