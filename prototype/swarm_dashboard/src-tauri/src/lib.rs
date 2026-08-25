mod commands;

use commands::agent_runtime::{
    launch_agent_runtime, list_agent_runtimes, stop_agent_runtime, AgentRuntimePanelState,
};
use commands::swarm_dashboard::{
    get_swarm_topology, query_derivation_trace, stream_phoneme_vector, SwarmDashboardState,
};

// Both command sets are registered in one `generate_handler!` call - Tauri's
// builder only accepts a single `invoke_handler`. `swarm_dashboard`'s own
// `register_swarm_commands()` helper (M2.10) is left as-is, unused here on
// purpose: it stays the concrete-over-Wry pattern that fixed the real
// E0277 that helper exists to document, without disturbing that
// already-CI-verified-green code path for this unrelated M2.8 addition.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(SwarmDashboardState::placeholder())
        .manage(AgentRuntimePanelState::default())
        .invoke_handler(tauri::generate_handler![
            get_swarm_topology,
            query_derivation_trace,
            stream_phoneme_vector,
            list_agent_runtimes,
            launch_agent_runtime,
            stop_agent_runtime,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauricode Swarm Dashboard");
}
