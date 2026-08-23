//! E2E against a REAL isolated swarm-node instance (no external mesh):
//! spawn -> check -> define -> explain -> claim -> complete -> verify,
//! plus fmt/convert over a fixture tasks.my.
//!
//! Requires SWARM_NODE_BIN env var pointing at a swarm-node binary
//! (e.g. /home/agents/GitHub/my-lisp/target/release/swarm-node).
//! Skipped (with a note) when unset — no fake substitutes.

use std::process::{Command, Stdio};
use std::time::Duration;

fn node_bin() -> Option<std::path::PathBuf> {
    std::env::var("SWARM_NODE_BIN").ok().map(std::path::PathBuf::from)
}

fn spawn_node(dir: &std::path::Path, port: u16) -> std::process::Child {
    Command::new(node_bin().unwrap())
        .arg("--port").arg(port.to_string())
        .arg("--node-id").arg("cli-e2e-node")
        .arg("--project").arg("e2e")
        .arg("--data-dir").arg(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn swarm-node")
}

fn cli(args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_swarm-cli"))
        .args(args)
        .output()
        .expect("run swarm-cli");
    let mut bytes = out.stdout.clone();
    bytes.extend_from_slice(&out.stderr);
    (String::from_utf8_lossy(&bytes).to_string(), out.status.code().unwrap_or(-1))
}

/// Grabs a free ephemeral port (bind :0 then release) so tests never
/// collide with live mesh services or each other.
fn free_port() -> u16 {
    std::net::TcpListener::bind(("127.0.0.1", 0))
        .expect("bind ephemeral")
        .local_addr()
        .expect("local addr")
        .port()
}

fn wait_listening(port: u16) -> bool {
    for _ in 0..40 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    false
}

#[test]
fn full_lifecycle_against_real_node() {
    let Some(bin) = node_bin() else {
        eprintln!("SWARM_NODE_BIN unset — e2e skipped");
        return;
    };
    assert!(bin.exists(), "SWARM_NODE_BIN points nowhere");
    let dir = std::env::temp_dir().join(format!("swarm-cli-e2e-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let port = free_port();
    let mut child = spawn_node(&dir.join("data"), port);
    assert!(wait_listening(port), "node did not start listening");
    let addr = format!("127.0.0.1:{port}");

    // check: isolated node has 0 peers => unhealthy exit 1, but valid JSON
    let (out, code) = cli(&["check", "--node", &addr]);
    assert_eq!(code, 1, "isolated node should be unhealthy: {out}");
    assert!(out.contains("\"synced\":false") || out.contains("\"healthy\":false"), "{out}");

    // define + explain round trip
    let (out, code) = cli(&[
        "task", "define", "E2E-DEMO", "--priority", "7", "--caps", "rust,lisp",
        "--desc", "demo task", "--node", &addr,
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("\"ok\":true"), "{out}");

    let (out, code) = cli(&["explain", "E2E-DEMO", "--repos", "/nonexistent", "--node", &addr]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("\"completed\":false"), "{out}");
    assert!(out.contains("claimable now"), "{out}");
    assert!(out.contains("\"priority\":7"), "{out}");

    // claim then complete with generation fencing
    let (out, code) = cli(&["task", "claim", "E2E-DEMO", "--node", &addr]);
    assert_eq!(code, 0, "{out}");
    let (out, code) = cli(&["task", "complete", "E2E-DEMO", "--gen", "1", "--node", &addr]);
    assert_eq!(code, 0, "{out}");
    let (out, _) = cli(&["explain", "E2E-DEMO", "--repos", "/nonexistent", "--node", &addr]);
    assert!(out.contains("\"completed\":true"), "{out}");

    // error path surfaces as ok:false + exit 5, never silent
    let (out, code) = cli(&["task", "complete", "E2E-DEMO", "--gen", "9", "--node", &addr]);
    assert_eq!(code, 5, "{out}");
    assert!(out.contains("\"ok\":false"), "{out}");

    child.kill().ok();
    // Reap the killed child so the OS doesn't leave a zombie behind.
    child.wait().ok();
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn fmt_normalizes_and_check_detects_drift() {
    let dir = std::env::temp_dir().join(format!("swarm-cli-fmt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("tasks.my");
    std::fs::write(
        &file,
        "((kind . tasks-my)\n (tasks . ((\"B\" . ((done . t))) (\"A\" . ((priority . 3)))))\n)",
    )
    .unwrap();
    let exe = env!("CARGO_BIN_EXE_swarm-cli");

    // fmt prints canonical text and reports count
    let out = Command::new(exe).args(["fmt", file.to_str().unwrap()]).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("(\"A\" . ("));
    assert!(text.contains("(priority . 3)"));

    // --check fails (exit 3) while file differs from canonical
    let out = Command::new(exe).args(["fmt", file.to_str().unwrap(), "--check"]).output().unwrap();
    assert_eq!(out.status.code(), Some(3));

    // apply canonical output, now --check passes (exit 0)
    let canonical = out2_stdout(&file, exe);
    std::fs::write(&file, &canonical).unwrap();
    let out = Command::new(exe).args(["fmt", file.to_str().unwrap(), "--check"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));

    std::fs::remove_dir_all(&dir).ok();
}

fn out2_stdout(file: &std::path::Path, exe: &str) -> Vec<u8> {
    Command::new(exe).args(["fmt", file.to_str().unwrap()]).output().unwrap().stdout
}
