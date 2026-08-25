//! M2.9 — second adapter proof.
//!
//! Run with: `cargo test --test second_adapter_proof --features mock,opencode`
//!
//! Deliberately NOT a Claude adapter: spawning a second, recursive AI
//! agent process as a subprocess is a materially different risk category
//! from spawning `opencode serve` (autonomous agent behavior, potential
//! runaway cost/loops) and was not something to decide unilaterally while
//! working solo. The plan's own stated alternative — "Claude or minimal
//! dummy second runtime" — is exercised here via the already-real,
//! already-proven-stable `OpenCodeAdapter` instead of a dummy: it is a
//! genuinely different, independently-implemented adapter (real
//! subprocess vs. `MockAdapter`'s pure in-memory state), which is a
//! stronger proof than a synthetic dummy would have been.
//!
//! This is an integration test (crate `tests/`, not a unit test inside
//! `src/`) specifically because it can only see this crate's *public*
//! API — a real black-box check that both adapters satisfy
//! `AgentRuntimeAdapter` from the outside, not just from implementation
//! details visible inside the crate.

use agent_runtime_contract::{AdapterError, AgentRuntimeAdapter, MockAdapter, OpenCodeAdapter, Status};

/// The exact same generic driver from M2.4's `mock::tests`, reproduced
/// here at the integration-test boundary (not `pub`-exported from the
/// crate itself — a driver like this belongs to whoever is testing the
/// contract, not to the contract's own public surface) to prove it works
/// unchanged against a second, independently-implemented adapter.
fn drive_full_lifecycle(adapter: &mut impl AgentRuntimeAdapter, cwd: &str) {
    let handle = adapter.launch(cwd, None).expect("launch should succeed");

    // Bounded poll rather than a single immediate check - OpenCodeAdapter
    // needs a brief real moment to start; MockAdapter is already Fresh
    // immediately, so the loop just exits on its first iteration for
    // that adapter.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    let mut confirmed_fresh = false;
    while std::time::Instant::now() < deadline {
        if adapter.status(&handle).expect("status should resolve") == Status::Fresh {
            confirmed_fresh = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(confirmed_fresh, "adapter never reached Fresh status");

    let workspace = adapter.workspace(&handle).expect("workspace should resolve");
    assert_eq!(workspace.cwd.as_deref(), Some(cwd));

    adapter.stop(&handle).expect("stop should succeed");
}

#[test]
fn mock_adapter_satisfies_the_contract() {
    let mut adapter = MockAdapter::new();
    drive_full_lifecycle(&mut adapter, "/tmp");
}

#[test]
fn opencode_adapter_satisfies_the_same_contract() {
    let mut adapter = OpenCodeAdapter::new();
    drive_full_lifecycle(&mut adapter, "/home/agents/GitHub/tauricode");
}

/// The actual point of "second adapter proof": one function, written
/// once, against the trait alone, genuinely drives two independently
/// implemented adapters — proving interchangeability isn't just an
/// unexercised claim.
#[test]
fn both_adapters_are_interchangeable_through_one_generic_call_site() {
    fn launch_and_confirm(adapter: &mut dyn AgentRuntimeAdapter, cwd: &str) -> Result<(), AdapterError> {
        let handle = adapter.launch(cwd, None)?;
        adapter.workspace(&handle)?;
        adapter.stop(&handle)?;
        Ok(())
    }

    let mut mock = MockAdapter::new();
    let mut opencode = OpenCodeAdapter::new();

    // Same function, same signature (dyn AgentRuntimeAdapter), two
    // concrete types passed through it - the call site itself never
    // names which adapter it's driving.
    launch_and_confirm(&mut mock, "/tmp").expect("mock adapter should satisfy the call site");
    launch_and_confirm(&mut opencode, "/home/agents/GitHub/tauricode")
        .expect("opencode adapter should satisfy the same call site");
}

/// The capability-flag differentiation (audit finding: attach/resume
/// semantics are runtime-specific, expressed as flags, never a fake
/// shared method) genuinely differs between these two real adapters -
/// not hypothetically, but as implemented.
#[test]
fn capability_flags_genuinely_differ_between_real_adapters() {
    let mock = MockAdapter::new();
    let opencode = OpenCodeAdapter::new();

    assert!(!mock.supports_live_attach());
    assert!(!mock.supports_resume());

    // Evidenced this session (opencode.log: multiple run= ids attaching
    // to one live session.id) - see opencode_adapter.rs's own doc
    // comment on supports_live_attach().
    assert!(opencode.supports_live_attach());
    assert!(!opencode.supports_resume());
}
