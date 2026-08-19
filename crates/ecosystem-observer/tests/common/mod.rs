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
