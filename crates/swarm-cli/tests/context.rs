use std::process::Command;

fn write_repo(root: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let dir = root.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("repo.my"), body).unwrap();
    dir
}

#[test]
fn context_card_parses_multiline_lists_and_counts_tasks() {
    let tmp = std::env::temp_dir().join(format!("swarm-cli-ctx-{}", std::process::id()));
    let dir = write_repo(
        &tmp,
        "demo",
        r#"(repository
  (id demo)
  (role tester-role)
  (exports
    thing-a
    thing-b)
  (imports language-contract)
  (capabilities rust docs)
  (authorities demo-stuff)
  (non-authorities everything-else))"#,
    );
    std::fs::write(
        dir.join("tasks.my"),
        "((kind . tasks-my)\n (tasks . ((\"T1\" . ((done . t))) (\"T2\" . ((done . ()))))))\n",
    )
    .unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_swarm-cli"))
        .args(["context", dir.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json = String::from_utf8_lossy(&out.stdout);
    assert!(json.contains("\"repo\":\"demo\""));
    assert!(json.contains("\"role\":\"tester-role\""));
    assert!(json.contains("\"exports\":[\"thing-a\",\"thing-b\"]")); // multiline list parsed
    assert!(json.contains("\"tasks_total\":2"));
    assert!(json.contains("\"tasks_done\":1"));

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn all_mode_covers_every_repo_with_repodotmy_and_skips_rest() {
    let tmp = std::env::temp_dir().join(format!("swarm-cli-ctx-all-{}", std::process::id()));
    write_repo(&tmp, "alpha", "(repository (id alpha) (role r))");
    write_repo(&tmp, "beta", "(repository (id beta) (role r))");
    std::fs::create_dir_all(tmp.join("gamma")).unwrap(); // no repo.my -> excluded

    let out = Command::new(env!("CARGO_BIN_EXE_swarm-cli"))
        .args(["context", "--all", "--root", tmp.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let json = String::from_utf8_lossy(&out.stdout);
    assert!(json.contains("\"count\":2"));
    assert!(json.contains("\"repo\":\"alpha\""));
    assert!(json.contains("\"repo\":\"beta\""));
    assert!(!json.contains("gamma"));

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn missing_repodotmy_is_an_error_not_a_guess() {
    let tmp = std::env::temp_dir().join(format!("swarm-cli-ctx-empty-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_swarm-cli"))
        .args(["context", tmp.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    std::fs::remove_dir_all(&tmp).ok();
}
