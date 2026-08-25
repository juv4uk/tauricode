//! M2.3 — the `AgentRuntimeAdapter` trait itself.
//!
//! Every method here traces back to one of the seven proven operations
//! from the adapter-contract audit. `send_input` and a `shutdown`
//! separate from `stop` are absent — no proven core need. `attach` is
//! absent as a base method too: the audit found OpenCode's "attach" (live
//! socket reconnect) and Claude's "resume" (cold disk resume) are
//! structurally different guarantees, not one operation under two names —
//! expressing that as a fake common method would hide exactly the
//! semantic gap the audit exists to surface. Instead, adapters declare
//! what they actually support via `supports_live_attach`/`supports_resume`
//! capability flags, so Core never has to assume a guarantee an adapter
//! doesn't actually provide.

use crate::events::Event;
use crate::types::{Capabilities, Identity, RuntimeHandle, Status, TaskId, Workspace};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    LaunchFailed(String),
    NotFound,
    Unsupported,
    Io(String),
}

pub trait AgentRuntimeAdapter {
    /// Start a new runtime instance. `task` is Tauricode's own
    /// `tasks.my`-scoped id, never a runtime-internal session id (audit:
    /// "task-id != runtime session-id") - the adapter is responsible for
    /// its own internal session bookkeeping, if it has one, without
    /// exposing it through this signature.
    fn launch(&mut self, cwd: &str, task: Option<TaskId>) -> Result<RuntimeHandle, AdapterError>;

    /// Terminate a running instance. No separate `shutdown` (graceful vs
    /// forced) - no evidence this distinction is needed yet.
    fn stop(&mut self, handle: &RuntimeHandle) -> Result<(), AdapterError>;

    /// Correlation/liveness status - see `types::Status` doc comment for
    /// why this is a freshness axis, not a trust axis, and why it is kept
    /// separate from `LifecycleState`.
    fn status(&self, handle: &RuntimeHandle) -> Result<Status, AdapterError>;

    /// Self-reported, therefore never authoritative on its own - same
    /// caveat as `ecosystem-observer::SelfReportedIdentity`.
    fn identity(&self, handle: &RuntimeHandle) -> Result<Identity, AdapterError>;

    /// What the adapter *claims* the runtime can do, not a verified
    /// grant.
    fn capabilities(&self, handle: &RuntimeHandle) -> Result<Capabilities, AdapterError>;

    /// OS-level cwd/repo association, matching `OsObservedFacts`.
    fn workspace(&self, handle: &RuntimeHandle) -> Result<Workspace, AdapterError>;

    /// Normalized events for this handle, already mapped from whatever
    /// raw vocabulary the underlying runtime actually speaks - the
    /// adapter's own job, never Core's.
    fn events(&self, handle: &RuntimeHandle) -> Result<Vec<Event>, AdapterError>;

    /// Whether this adapter can reconnect to an already-running instance
    /// via a live connection (OpenCode's model). Defaults to `false` so a
    /// new adapter never silently claims a guarantee it hasn't actually
    /// implemented.
    fn supports_live_attach(&self) -> bool {
        false
    }

    /// Whether this adapter can resume a previously-stopped instance from
    /// durable storage (Claude Code's `-r`/`--continue` model). Defaults
    /// to `false` for the same reason.
    fn supports_resume(&self) -> bool {
        false
    }
}
