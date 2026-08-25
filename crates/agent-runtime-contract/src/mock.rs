//! M2.4 — mock adapter. First (and, for this session, only — see
//! docs/STAGE2-RUNTIME-ADAPTER-PLAN.md's "Why this increment order")
//! implementor of `AgentRuntimeAdapter`. Purely in-memory, no subprocess,
//! no I/O — proves the trait is actually implementable and drivable
//! without a caller knowing it's fake.
//!
//! Gated behind `cfg(any(test, feature = "mock"))` on its `mod mock;`
//! declaration in `lib.rs`, so it stays available to other crates that
//! want it later (e.g. a scheduler/dashboard integration test) without
//! becoming default production surface of this crate.

use crate::adapter::{AdapterError, AgentRuntimeAdapter};
use crate::events::Event;
use crate::lifecycle::{HandleRegistry, LifecycleState};
use crate::types::{Capabilities, Identity, RuntimeHandle, Status, TaskId, Workspace};
use std::collections::HashMap;

#[derive(Debug, Default)]
struct HandleState {
    identity: Identity,
    capabilities: Capabilities,
    workspace: Workspace,
    status: Status,
    events: Vec<Event>,
}

#[derive(Debug, Default)]
pub struct MockAdapter {
    registry: HandleRegistry,
    states: HashMap<RuntimeHandle, HandleState>,
    next_id: u64,
    supports_live_attach: bool,
    supports_resume: bool,
}

impl MockAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Test-only configuration: builder-style flag setters, since a real
    /// adapter's capability flags are fixed by what it actually
    /// implements, not runtime-mutable in production.
    pub fn with_live_attach(mut self, value: bool) -> Self {
        self.supports_live_attach = value;
        self
    }

    pub fn with_resume(mut self, value: bool) -> Self {
        self.supports_resume = value;
        self
    }

    /// Test-only: script the event sequence a given handle's `events()`
    /// call will return.
    pub fn script_events(&mut self, handle: &RuntimeHandle, events: Vec<Event>) {
        if let Some(state) = self.states.get_mut(handle) {
            state.events = events;
        }
    }
}

impl AgentRuntimeAdapter for MockAdapter {
    fn launch(&mut self, cwd: &str, task: Option<TaskId>) -> Result<RuntimeHandle, AdapterError> {
        self.next_id += 1;
        let handle = RuntimeHandle::new(format!("mock-{}", self.next_id));

        self.registry
            .register(handle.clone())
            .map_err(|e| AdapterError::LaunchFailed(format!("{e:?}")))?;
        self.registry
            .transition(&handle, LifecycleState::Running)
            .map_err(|e| AdapterError::LaunchFailed(format!("{e:?}")))?;

        let mut state = HandleState {
            workspace: Workspace {
                cwd: Some(cwd.to_string()),
                repo_association: None,
            },
            ..Default::default()
        };
        state.identity.task = task.map(|t| t.as_str().to_string());
        self.states.insert(handle.clone(), state);

        Ok(handle)
    }

    fn stop(&mut self, handle: &RuntimeHandle) -> Result<(), AdapterError> {
        self.registry
            .transition(handle, LifecycleState::Stopped)
            .map_err(|_| AdapterError::NotFound)
    }

    fn status(&self, handle: &RuntimeHandle) -> Result<Status, AdapterError> {
        self.states
            .get(handle)
            .map(|s| s.status)
            .ok_or(AdapterError::NotFound)
    }

    fn identity(&self, handle: &RuntimeHandle) -> Result<Identity, AdapterError> {
        self.states
            .get(handle)
            .map(|s| s.identity.clone())
            .ok_or(AdapterError::NotFound)
    }

    fn capabilities(&self, handle: &RuntimeHandle) -> Result<Capabilities, AdapterError> {
        self.states
            .get(handle)
            .map(|s| s.capabilities.clone())
            .ok_or(AdapterError::NotFound)
    }

    fn workspace(&self, handle: &RuntimeHandle) -> Result<Workspace, AdapterError> {
        self.states
            .get(handle)
            .map(|s| s.workspace.clone())
            .ok_or(AdapterError::NotFound)
    }

    fn events(&self, handle: &RuntimeHandle) -> Result<Vec<Event>, AdapterError> {
        self.states
            .get(handle)
            .map(|s| s.events.clone())
            .ok_or(AdapterError::NotFound)
    }

    fn supports_live_attach(&self) -> bool {
        self.supports_live_attach
    }

    fn supports_resume(&self) -> bool {
        self.supports_resume
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{OutputKind, ToolStatus};

    /// Drives any `AgentRuntimeAdapter` through a full lifecycle without
    /// referencing `MockAdapter` (or any concrete adapter type) by name
    /// in its signature. This is the actual check for "Core does not
    /// need to know which runtime is under it" — a generic function, not
    /// a claim.
    fn drive_full_lifecycle(adapter: &mut impl AgentRuntimeAdapter) -> RuntimeHandle {
        let handle = adapter
            .launch("/home/agents/GitHub/tauricode", Some(TaskId::new("T-1")))
            .expect("launch should succeed");

        let status = adapter.status(&handle).expect("status should resolve");
        assert_eq!(status, Status::Fresh);

        let workspace = adapter.workspace(&handle).expect("workspace should resolve");
        assert_eq!(
            workspace.cwd.as_deref(),
            Some("/home/agents/GitHub/tauricode")
        );

        adapter.stop(&handle).expect("stop should succeed");

        handle
    }

    #[test]
    fn full_lifecycle_via_generic_trait_bound() {
        let mut adapter = MockAdapter::new();
        let handle = drive_full_lifecycle(&mut adapter);

        // Post-stop: unknown ops on this handle behave per contract, not
        // by panic. status()/identity()/etc. still resolve (state kept),
        // but a second stop() is rejected by the lifecycle registry.
        let err = adapter.stop(&handle).unwrap_err();
        assert_eq!(err, AdapterError::NotFound);
    }

    #[test]
    fn scripted_events_are_returned_in_order() {
        let mut adapter = MockAdapter::new();
        let handle = adapter.launch("/tmp/x", None).unwrap();

        let scripted = vec![
            Event::RuntimeStarted {
                handle: handle.clone(),
                pid: Some(999),
                cwd: Some("/tmp/x".to_string()),
            },
            Event::ToolStarted {
                handle: handle.clone(),
                task_id: None,
                tool_name: "bash".to_string(),
            },
            Event::ToolCompleted {
                handle: handle.clone(),
                task_id: None,
                tool_name: "bash".to_string(),
                status: ToolStatus::Ok,
                result_ref: None,
            },
            Event::OutputChunk {
                handle: handle.clone(),
                task_id: None,
                output_kind: OutputKind::Text,
                content: "done".to_string(),
            },
        ];
        adapter.script_events(&handle, scripted.clone());

        let observed = adapter.events(&handle).unwrap();
        assert_eq!(observed, scripted);
    }

    #[test]
    fn unknown_handle_returns_typed_error_not_panic() {
        let adapter = MockAdapter::new();
        let bogus = RuntimeHandle::new("never-launched");
        assert_eq!(adapter.status(&bogus).unwrap_err(), AdapterError::NotFound);
        assert_eq!(
            adapter.identity(&bogus).unwrap_err(),
            AdapterError::NotFound
        );
        assert_eq!(adapter.events(&bogus).unwrap_err(), AdapterError::NotFound);
    }

    #[test]
    fn capability_flags_default_false_until_configured() {
        let plain = MockAdapter::new();
        assert!(!plain.supports_live_attach());
        assert!(!plain.supports_resume());

        let configured = MockAdapter::new().with_live_attach(true).with_resume(true);
        assert!(configured.supports_live_attach());
        assert!(configured.supports_resume());
    }
}
