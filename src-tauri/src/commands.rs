//! Tauri commands = the public API between UI and backend. Keep them thin:
//! load the environment, delegate to `instance` / `logs` / `config`.

use crate::config::{self, Environment, Workspace};
use crate::instance::{self, State, Transitions};
use crate::logs::{self, LogChunk};
use std::path::Path;
use std::process::Command;
use tauri::{AppHandle, State as TauriState};

#[tauri::command]
pub fn list_workspaces(app: AppHandle) -> Result<Vec<Workspace>, String> {
    config::load_workspaces(&app)
}

/// Insert or update (matched by id).
#[tauri::command]
pub fn save_workspace(app: AppHandle, workspace: Workspace) -> Result<(), String> {
    let mut all = config::load_workspaces(&app)?;
    match all.iter_mut().find(|w| w.id == workspace.id) {
        Some(slot) => *slot = workspace,
        None => all.push(workspace),
    }
    config::store_workspaces(&app, &all)
}

/// Removes the workspace and the environment entries inside it (files on disk are untouched).
#[tauri::command]
pub fn delete_workspace(app: AppHandle, id: String) -> Result<(), String> {
    let mut all = config::load_workspaces(&app)?;
    all.retain(|w| w.id != id);
    config::store_workspaces(&app, &all)?;
    let mut envs = config::load(&app)?;
    envs.retain(|e| e.workspace_id != id);
    config::store(&app, &envs)
}

#[tauri::command]
pub fn list_environments(app: AppHandle) -> Result<Vec<Environment>, String> {
    config::load(&app)
}

/// JAR files found directly in `dir`, for the "choose AEM jar" step of the create dialog.
#[tauri::command]
pub fn scan_jars(dir: String) -> Vec<String> {
    instance::scan_jars(&dir)
}

/// Insert or update (matched by id). If `env.jar` is an absolute path outside `env.dir`,
/// the file is copied into `env.dir` and `env.jar` becomes its file name.
/// `async` so copying a large jar does not block the UI thread.
#[tauri::command]
pub async fn save_environment(app: AppHandle, mut env: Environment) -> Result<(), String> {
    let jar = Path::new(&env.jar);
    if jar.is_absolute() {
        let name = jar.file_name().ok_or("Invalid JAR path")?.to_owned();
        let dest = Path::new(&env.dir).join(&name);
        std::fs::create_dir_all(&env.dir).map_err(|e| e.to_string())?;
        let same = dest.exists() && std::fs::canonicalize(&dest).ok() == std::fs::canonicalize(jar).ok();
        if !same {
            std::fs::copy(jar, &dest).map_err(|e| format!("Could not copy JAR to {}: {e}", dest.display()))?;
        }
        env.jar = name.to_string_lossy().into_owned();
    }
    let mut all = config::load(&app)?;
    match all.iter_mut().find(|e| e.id == env.id) {
        Some(slot) => *slot = env,
        None => all.push(env),
    }
    config::store(&app, &all)
}

#[tauri::command]
pub fn delete_environment(app: AppHandle, id: String) -> Result<(), String> {
    let mut all = config::load(&app)?;
    all.retain(|e| e.id != id);
    config::store(&app, &all)
}

#[tauri::command]
pub fn get_status(app: AppHandle, tr: TauriState<Transitions>, id: String) -> Result<State, String> {
    Ok(instance::status(&config::find(&app, &id)?, &tr))
}

#[tauri::command]
pub fn start_environment(app: AppHandle, tr: TauriState<Transitions>, id: String) -> Result<(), String> {
    instance::start(&config::find(&app, &id)?, &tr)
}

#[tauri::command]
pub async fn stop_environment(app: AppHandle, tr: TauriState<'_, Transitions>, id: String) -> Result<(), String> {
    instance::stop(&config::find(&app, &id)?, &tr)
}

#[tauri::command]
pub fn list_logs(app: AppHandle, id: String) -> Result<Vec<String>, String> {
    Ok(logs::list(&config::find(&app, &id)?))
}

#[tauri::command]
pub fn read_log(app: AppHandle, id: String, file: String, offset: Option<u64>) -> Result<LogChunk, String> {
    logs::read(&config::find(&app, &id)?, &file, offset)
}

/// Opens a URL (http/https only) in the default browser.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("Only http(s) URLs are allowed".into());
    }
    open_with_os(&url)
}

/// Shows a folder in the OS file manager.
#[tauri::command]
pub fn reveal_path(path: String) -> Result<(), String> {
    open_with_os(&path)
}

fn open_with_os(target: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut c = Command::new("open");
    #[cfg(target_os = "linux")]
    let mut c = Command::new("xdg-open");
    #[cfg(target_os = "windows")]
    let mut c = {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", ""]);
        c
    };
    c.arg(target).spawn().map(|_| ()).map_err(|e| e.to_string())
}
