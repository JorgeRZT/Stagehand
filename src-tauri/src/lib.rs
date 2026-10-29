//! Stagehand – backend entry point.
//!
//! Layout (see docs/ARCHITECTURE.md):
//! - `config`   : persisted list of environments (JSON in the OS config dir)
//! - `instance` : start / stop / status of one AEM instance (the only place that spawns processes)
//! - `logs`     : discovery and incremental reading of log files
//! - `commands` : thin `#[tauri::command]` layer exposed to the UI

mod commands;
mod config;
mod instance;
mod logs;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(instance::Transitions::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_workspaces,
            commands::save_workspace,
            commands::delete_workspace,
            commands::list_environments,
            commands::scan_jars,
            commands::save_environment,
            commands::delete_environment,
            commands::get_status,
            commands::start_environment,
            commands::stop_environment,
            commands::list_logs,
            commands::read_log,
            commands::open_url,
            commands::reveal_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Stagehand");
}
