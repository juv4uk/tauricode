//! M2.5 — OpenCode adapter, minimal launch/stop/status only, as scoped in
//! `docs/STAGE2-RUNTIME-ADAPTER-PLAN.md`. Deliberately does NOT implement
//! real identity/capabilities/workspace introspection or event
//! normalization (M2.6) — those return honest stubs here, not invented
//! data.
//!
//! Gated behind `feature = "opencode"` (same pattern as `mock`'s
//! `cfg(any(test, feature = "mock"))`) so a default build never spawns a
//! real subprocess. Preflight for this increment (per the plan, done
//! before writing this file, this session): `uptime` 57min stable,
//! load average 0.05, 6.3GB RAM available, `ollama.service` confirmed
//! inactive/disabled.
//!
//! Deliberate safety choices, given this session's own WSL-freeze
//! history:
//! - Spawns an isolated `opencode serve` instance on its own port, never
//!   the shared always-on `opencode-shared.service` — no interference
//!   with, or dependency on, that process.
//! - `stop()` uses a bounded poll loop on `try_wait()`, never a blocking
//!   `wait()` with no timeout.
//! - No HTTP round-trip to the spawned server anywhere in this file or
//!   its tests — liveness is checked at the OS-process level only
//!   (`try_wait()`), the same provenance class `ecosystem-observer`
//!   already uses for "OS-observed" facts.

#![cfg(feature = "opencode")]

use crate::adapter::{AdapterError, AgentRuntimeAdapter};
use crate::events::Event;
use crate::types::{Capabilities, Identity, RuntimeHandle, Status, TaskId, Workspace};
use std::collections::HashMap;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

struct Instance {
    child: Child,
    cwd: String,
    /// M2.7: the Tauricode `TaskId` this instance was launched for, if
    /// any — kept as our own string, never a runtime session id (audit:
    /// "task-id != runtime session-id"). `origin` (audit's `TaskBound`
    /// field) is not tracked yet - no caller of `launch()` this session
    /// has anything to put there, so it stays absent rather than
    /// invented.
    task_id: Option<TaskId>,
}

#[derive(Default)]
pub struct OpenCodeAdapter {
    instances: HashMap<RuntimeHandle, Instance>,
    next_id: u64,
    next_port: u16,
}

impl OpenCodeAdapter {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
            next_id: 0,
            // Deliberately far from the shared service's :4097 and any
            // well-known port, and incremented per launch within one
            // adapter instance so parallel launches in the same test
            // process don't collide. A production version would probe
            // for a genuinely free port; documented here as a known
            // limitation, not silently pretended away.
            next_port: 39901,
        }
    }

    fn bounded_wait_exit(child: &mut Child, timeout: Duration) -> Result<(), AdapterError> {
        let deadline = Instant::now() + timeout;
        loop {
            match child.try_wait() {
                Ok(Some(_)) => return Ok(()),
                Ok(None) => {
                    if Instant::now() >= deadline {
                        return Err(AdapterError::Io(
                            "process did not exit within bounded wait".to_string(),
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => return Err(AdapterError::Io(e.to_string())),
            }
        }
    }
}

impl AgentRuntimeAdapter for OpenCodeAdapter {
    fn launch(&mut self, cwd: &str, task: Option<TaskId>) -> Result<RuntimeHandle, AdapterError> {
        let port = self.next_port;
        self.next_port += 1;

        let child = Command::new("opencode")
            .args([
                "serve",
                "--hostname",
                "127.0.0.1",
                "--port",
                &port.to_string(),
            ])
            .current_dir(cwd)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| AdapterError::LaunchFailed(e.to_string()))?;

        self.next_id += 1;
        let handle = RuntimeHandle::new(format!("opencode-{}", self.next_id));
        self.instances.insert(
            handle.clone(),
            Instance {
                child,
                cwd: cwd.to_string(),
                task_id: task,
            },
        );
        Ok(handle)
    }

    fn stop(&mut self, handle: &RuntimeHandle) -> Result<(), AdapterError> {
        let instance = self.instances.get_mut(handle).ok_or(AdapterError::NotFound)?;
        // kill() itself is non-blocking (just sends the signal); the
        // bounded wait below is what actually protects against a hang.
        instance
            .child
            .kill()
            .map_err(|e| AdapterError::Io(e.to_string()))?;
        Self::bounded_wait_exit(&mut instance.child, Duration::from_secs(3))?;
        self.instances.remove(handle);
        Ok(())
    }

    fn status(&self, handle: &RuntimeHandle) -> Result<Status, AdapterError> {
        let instance = self.instances.get(handle).ok_or(AdapterError::NotFound)?;
        // Checked via /proc, not Child::try_wait() - the trait's &self
        // signature can't call try_wait (it needs &mut), and a
        // liveness-only check is exactly the OS-observed provenance this
        // status is supposed to represent anyway (see PID_LIVENESS doc
        // comment below).
        let pid_alive = pid_is_alive(instance.child.id());
        Ok(if pid_alive { Status::Fresh } else { Status::Orphaned })
    }

    fn identity(&self, handle: &RuntimeHandle) -> Result<Identity, AdapterError> {
        let instance = self.instances.get(handle).ok_or(AdapterError::NotFound)?;
        // M2.7: `task` is the one field this adapter can honestly fill in
        // (it's what launch() was actually given). model/role/instance
        // remain None - no real self-reported-identity introspection is
        // wired yet, and this method never invents a value for a field
        // it has no evidence for.
        Ok(Identity {
            task: instance.task_id.as_ref().map(|t| t.as_str().to_string()),
            ..Identity::default()
        })
    }

    fn capabilities(&self, handle: &RuntimeHandle) -> Result<Capabilities, AdapterError> {
        self.instances.get(handle).ok_or(AdapterError::NotFound)?;
        Ok(Capabilities::default())
    }

    fn workspace(&self, handle: &RuntimeHandle) -> Result<Workspace, AdapterError> {
        let instance = self.instances.get(handle).ok_or(AdapterError::NotFound)?;
        Ok(Workspace {
            cwd: Some(instance.cwd.clone()),
            repo_association: None,
        })
    }

    fn events(&self, handle: &RuntimeHandle) -> Result<Vec<Event>, AdapterError> {
        self.instances.get(handle).ok_or(AdapterError::NotFound)?;
        // Honest stub: event normalization is M2.6, not this increment.
        Ok(Vec::new())
    }

    fn supports_live_attach(&self) -> bool {
        // OpenCode's real, observed model this session (opencode.log:
        // multiple `run=` ids attaching to one `session.id` over a live
        // server) - true is evidenced, not assumed.
        true
    }

    fn supports_resume(&self) -> bool {
        false
    }
}

/// OS-level liveness check via `/proc/<pid>` existence — same provenance
/// class as `ecosystem-observer`'s OS-observed facts. No `unsafe {}`
/// block; ordinary safe Rust.
///
/// Known, disclosed limitation: this alone cannot distinguish "our
/// child is still alive" from "the PID was reused by an unrelated
/// process after ours exited" — `ecosystem-observer`'s own
/// `identity_contract` module solves this properly with a
/// `process_start_token` (the kernel's per-PID start-time counter). Not
/// ported here because M2.5's scope is launch/stop/status only and the
/// test lifetime (seconds) makes reuse practically negligible; a real
/// production adapter should reuse that existing, proven mechanism
/// rather than re-solve it.
fn pid_is_alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bounded end-to-end: spawn a real, isolated opencode serve
    /// instance, confirm OS-level liveness, stop it, confirm it's gone.
    /// No HTTP call anywhere in this test - liveness is process-level
    /// only, by design (see module doc comment).
    #[test]
    fn launch_then_stop_real_opencode_process() {
        let mut adapter = OpenCodeAdapter::new();
        let handle = adapter
            .launch(
                "/home/agents/GitHub/tauricode",
                Some(TaskId::new("STAGE2-M2.7-TEST")),
            )
            .expect("launch should succeed - opencode binary confirmed present");

        // M2.7: task binding round-trips through identity() - our own
        // task id, never a runtime session id.
        let identity = adapter.identity(&handle).unwrap();
        assert_eq!(identity.task.as_deref(), Some("STAGE2-M2.7-TEST"));

        // Give the process a brief moment to actually start before the
        // first liveness check - bounded, not a blind sleep-and-hope.
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut confirmed_alive = false;
        while Instant::now() < deadline {
            if adapter.status(&handle).unwrap() == Status::Fresh {
                confirmed_alive = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(confirmed_alive, "opencode process never reached Fresh status");

        let workspace = adapter.workspace(&handle).unwrap();
        assert_eq!(
            workspace.cwd.as_deref(),
            Some("/home/agents/GitHub/tauricode")
        );

        adapter.stop(&handle).expect("stop should succeed within bounded wait");

        // Handle removed after stop - further ops return NotFound, not a
        // stale "still running" answer.
        assert_eq!(adapter.status(&handle).unwrap_err(), AdapterError::NotFound);
    }

    #[test]
    fn stop_unknown_handle_returns_typed_error() {
        let mut adapter = OpenCodeAdapter::new();
        let bogus = RuntimeHandle::new("never-launched");
        assert_eq!(adapter.stop(&bogus).unwrap_err(), AdapterError::NotFound);
    }

    #[test]
    fn launch_without_task_leaves_identity_task_none() {
        let mut adapter = OpenCodeAdapter::new();
        let handle = adapter
            .launch("/home/agents/GitHub/tauricode", None)
            .expect("launch should succeed");

        let identity = adapter.identity(&handle).unwrap();
        assert_eq!(identity.task, None);

        adapter.stop(&handle).expect("stop should succeed");
    }
}
