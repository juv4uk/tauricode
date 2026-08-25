mod commands;

use commands::swarm_dashboard::{register_swarm_commands, SwarmDashboardState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(SwarmDashboardState::placeholder())
        .invoke_handler(register_swarm_commands())
        .run(tauri::generate_context!())
        .expect("error while running Tauricode Swarm Dashboard");
}
