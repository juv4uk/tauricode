mod commands;

use commands::snapshot::get_ecosystem_snapshot;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_ecosystem_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running Tauricode agent-runtime-ui");
}
