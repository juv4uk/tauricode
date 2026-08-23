//! End-to-end: fixture repos on disk → routing plan, exactly what the CLI does.

use std::fs;
use std::process::Command;

fn write_repo(root: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let dir = root.join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("tasks.my"), body).unwrap();
    dir
}

#[test]
fn end_to_end_over_fixture_repos_json_and_text() {
    let tmp = std::env::temp_dir().join(format!("eco-sched-e2e-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();

    write_repo(
        &tmp,
        "my-lisp",
        r#"
; durable plan
((kind . tasks-my)
 (tasks .
  (("MY-CORE" . ((priority . 9.0) (capabilities . (rust lisp)) (done . t)))
   ("MY-NEXT" . ((priority . 5.0) (capabilities . (rust)) (depends-on . ("CML-BASE")))))))
"#,
    );
    write_repo(
        &tmp,
        "cml",
        r#"
((kind . tasks-my)
 (tasks .
  (("CML-BASE" . ((priority . 8.0) (origin . cml))))
  ))
"#,
    );
    // a repo without tasks.my must be ignored by the github-root scan
    fs::create_dir_all(tmp.join("empty-repo")).unwrap();

    let exe = env!("CARGO_BIN_EXE_ecosystem-scheduler");
    let out = Command::new(exe)
        .arg("--github-root")
        .arg(&tmp)
        .arg("--agent")
        .arg("rusty-1=rust,cml")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let json = String::from_utf8(out.stdout).unwrap();
    // empty-repo absent, both real repos present with counts
    assert!(json.contains("\"repo\":\"my-lisp\""));
    assert!(json.contains("\"tasks\":2"));
    assert!(json.contains("\"repo\":\"cml\""));
    assert!(json.contains("\"tasks\":1"));
    // CML-BASE ready and routed to rusty-1 (matches origin cml capability? no — caps empty => generic)
    assert!(json.contains("\"id\":\"CML-BASE\""));
    // MY-NEXT waits on CML-BASE -> in blocked_detail as waiting-on
    assert!(json.contains("\"kind\":\"waiting-on\",\"dep\":\"CML-BASE\""));
    // done task not routed
    assert!(!json.contains("\"id\":\"MY-CORE\""));

    let txt = Command::new(exe)
        .arg("--github-root")
        .arg(&tmp)
        .arg("--format")
        .arg("text")
        .output()
        .unwrap();
    assert!(txt.status.success());
    let text = String::from_utf8(txt.stdout).unwrap();
    assert!(text.contains("graph: 3 tasks | 1 done | 1 ready | 1 blocked"));
    assert!(text.contains("== _any (caps: any) =="));
    assert!(text.contains("waiting on `CML-BASE`"));

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn parse_error_reports_file_and_exit_code_3() {
    let tmp = std::env::temp_dir().join(format!("eco-sched-bad-{}", std::process::id()));
    write_repo(&tmp, "broken", "((kind . tasks-my)"); // unclosed list
    let exe = env!("CARGO_BIN_EXE_ecosystem-scheduler");
    let out = Command::new(exe).arg("--repo").arg(tmp.join("broken")).output().unwrap();
    assert_eq!(out.status.code(), Some(3));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("parse error"), "stderr was: {err}");
    assert!(err.contains("broken/tasks.my"), "stderr was: {err}");
    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn duplicate_ids_across_repos_produce_visible_warning() {
    let tmp = std::env::temp_dir().join(format!("eco-sched-dup-{}", std::process::id()));
    write_repo(
        &tmp,
        "a",
        "((kind . tasks-my) (tasks . ((\"X\" . ((priority . 5))))))",
    );
    write_repo(
        &tmp,
        "b",
        "((kind . tasks-my) (tasks . ((\"X\" . ((priority . 7))))))",
    );
    let exe = env!("CARGO_BIN_EXE_ecosystem-scheduler");
    let out = Command::new(exe)
        .arg("--github-root")
        .arg(&tmp)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    assert!(out.status.success());
    let json = String::from_utf8(out.stdout).unwrap();
    assert!(json.contains("duplicate task id `X` also defined in `b` (kept `a`"));
    fs::remove_dir_all(&tmp).ok();
}
