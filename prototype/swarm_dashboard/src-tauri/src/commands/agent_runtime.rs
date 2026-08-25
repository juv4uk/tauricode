// ============================================================================
// Tauricode Desktop Workbench - Agent Runtime Status Panel (M2.8, rescoped)
// Location: src-tauri/src/commands/agent_runtime.rs
// ============================================================================
//
// This is a deliberately separate panel from `swarm_dashboard.rs`'s
// `SwarmMeshTopology` - the original M2.8 plan item ("wire
// agent-runtime-contract behind the swarm-dashboard IPC commands") was
// rejected because `AgentRuntimeAdapter` (a launched process's lifecycle -
// launch/stop/status/identity/capabilities/workspace) and swarm mesh
// topology (registered nodes/connections/derivation traces/phoneme
// vectors) are different domains with no natural connection point. This
// file exposes the adapter-contract status shape on its own, without
// substituting or feeding into the mesh topology the other panel shows.
//
// Backed by `MockAdapter` (the "mock" feature), not the real,
// subprocess-spawning `OpenCodeAdapter` ("opencode" feature) - launching
// real agent processes from a GUI button is a separate, larger decision
// than exposing the status shape a launched runtime would have.

use agent_runtime_contract::{
    AdapterError, AgentRuntimeAdapter, Capabilities, Identity, MockAdapter, RuntimeHandle,
    Status, TaskId, Workspace,
};
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

/// One tracked runtime's current status, assembled from the adapter's
/// per-handle queries. `AgentRuntimeAdapter` has no "list all handles"
/// operation (the M2.3 audit found no proven need for one on the trait
/// itself) - so this panel's own state tracks which handles it launched,
/// the same way any real caller would.
#[derive(Debug, Clone, Serialize)]
pub struct AgentRuntimeSummary {
    pub handle: String,
    pub status: Status,
    pub identity: Identity,
    pub capabilities: Capabilities,
    pub workspace: Workspace,
}

#[derive(Default)]
pub struct AgentRuntimePanelState {
    adapter: Mutex<MockAdapter>,
    handles: Mutex<Vec<RuntimeHandle>>,
}

fn to_message(err: AdapterError) -> String {
    format!("{err:?}")
}

/// IPC Command: list every runtime this panel has launched, with its
/// current status/identity/capabilities/workspace.
#[tauri::command]
pub async fn list_agent_runtimes(
    state: State<'_, AgentRuntimePanelState>,
) -> Result<Vec<AgentRuntimeSummary>, String> {
    let adapter = state.adapter.lock().map_err(|e| e.to_string())?;
    let handles = state.handles.lock().map_err(|e| e.to_string())?;

    handles
        .iter()
        .map(|handle| {
            Ok(AgentRuntimeSummary {
                handle: handle.as_str().to_string(),
                status: adapter.status(handle).map_err(to_message)?,
                identity: adapter.identity(handle).map_err(to_message)?,
                capabilities: adapter.capabilities(handle).map_err(to_message)?,
                workspace: adapter.workspace(handle).map_err(to_message)?,
            })
        })
        .collect()
}

/// IPC Command: launch a new (mock) runtime instance for this panel to
/// track. `task` is Tauricode's own task id, per the contract's own
/// task-id != runtime-session-id distinction.
#[tauri::command]
pub async fn launch_agent_runtime(
    cwd: String,
    task: Option<String>,
    state: State<'_, AgentRuntimePanelState>,
) -> Result<String, String> {
    let mut adapter = state.adapter.lock().map_err(|e| e.to_string())?;
    let handle = adapter
        .launch(&cwd, task.map(TaskId::new))
        .map_err(to_message)?;

    let mut handles = state.handles.lock().map_err(|e| e.to_string())?;
    handles.push(handle.clone());

    Ok(handle.as_str().to_string())
}

/// IPC Command: stop a tracked runtime instance by its handle string.
#[tauri::command]
pub async fn stop_agent_runtime(
    handle: String,
    state: State<'_, AgentRuntimePanelState>,
) -> Result<(), String> {
    let mut adapter = state.adapter.lock().map_err(|e| e.to_string())?;
    adapter.stop(&RuntimeHandle::new(handle)).map_err(to_message)
}
