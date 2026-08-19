//! Test-only fixtures: real `git init`-ed temp repositories, not mocks.
//! Deliberately dependency-free (no `tempfile` crate) to keep this crate's
//! empty-`[dependencies]`/`[dev-dependencies]` convention.

use std::path::{Path, PathBuf};
use std::process::Command;

pub struct TempDir {
    pub path: PathBuf,
}

impl TempDir {
    pub fn new(label: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ecosystem-observer-test-{label}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir fixture");
        TempDir { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

pub fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .expect("failed to spawn git in test fixture");
    assert!(status.success(), "git {args:?} failed in {dir:?}");
}

/// `-b main` pins the initial branch name explicitly so tests don't depend
/// on the host's `init.defaultBranch` config (which varies).
pub fn init_repo(dir: &Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "test@example.invalid"]);
    git(dir, &["config", "user.name", "Test"]);
}

pub fn commit_all(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
}

pub fn head_sha(dir: &Path) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("failed to spawn git rev-parse in test fixture");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn init_bare_repo(dir: &Path) {
    let status = Command::new("git")
        .arg("init")
        .arg("-q")
        .arg("--bare")
        .arg("-b")
        .arg("main")
        .arg(dir)
        .status()
        .expect("failed to spawn git init --bare in test fixture");
    assert!(status.success());
}

/// `protocol.file.allow=always` is required by modern git for `submodule
/// add` with a local (non-URL) path — without it, git refuses for
/// security reasons unrelated to what this fixture is testing.
pub fn submodule_add(superproject: &Path, target_path: &str, name: &str) {
    let status = Command::new("git")
        .arg("-C")
        .arg(superproject)
        .args(["-c", "protocol.file.allow=always", "submodule", "add", "-q"])
        .arg(target_path)
        .arg(name)
        .status()
        .expect("failed to spawn git submodule add in test fixture");
    assert!(status.success());
}

pub fn clone_repo(source: &Path, dest: &Path) {
    let status = Command::new("git")
        .arg("clone")
        .arg("-q")
        .arg(source)
        .arg(dest)
        .status()
        .expect("failed to spawn git clone in test fixture");
    assert!(status.success());
}

pub fn worktree_add(repo: &Path, worktree_path: &Path, branch: &str) {
    git(
        repo,
        &[
            "worktree",
            "add",
            "-q",
            worktree_path.to_str().expect("non-utf8 test path"),
            branch,
        ],
    );
}

/// Named pipe (FIFO) with nothing ever writing to it. When a repo's
/// `.git/config` has `include.path` pointing at this, *any* git
/// invocation against that repo hangs while loading its config — before
/// reaching any subcommand-specific logic. Verified empirically (not
/// assumed) against git 2.43.0 before use: `git status` against such a
/// repo does not return on its own; an outer `timeout` was required to
/// reclaim it. This is a real, deterministic hang using only standard
/// git/POSIX mechanisms — no custom scripts, no env/PATH overrides on
/// the test process itself (which would risk interfering with other
/// tests running in parallel in the same process).
pub fn make_repo_with_hanging_config(dir: &Path) {
    init_repo(dir);
    let fifo = dir.join("hang.fifo");
    let status = Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("failed to spawn mkfifo in test fixture");
    assert!(status.success());
    git(
        dir,
        &[
            "config",
            "include.path",
            fifo.to_str().expect("non-utf8 test path"),
        ],
    );
}
