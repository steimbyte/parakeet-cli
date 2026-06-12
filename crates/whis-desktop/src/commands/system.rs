//! System Utility Commands

use crate::state::AppState;
use tauri::{AppHandle, State};
use whis_core::{AutotypeToolStatus, get_autotype_tool_status};

/// Get the command to toggle recording from an external source (e.g., GNOME custom shortcut).
#[tauri::command]
pub fn get_toggle_command() -> String {
    if std::path::Path::new("/.flatpak-info").exists() {
        return "flatpak run dev.steimbyte.parakeet-cli --toggle".to_string();
    }

    if let Ok(appimage_path) = std::env::var("APPIMAGE") {
        return format!("{} --toggle", appimage_path);
    }

    if let Ok(exe_path) = std::env::current_exe()
        && let Ok(canonical) = exe_path.canonicalize()
    {
        return format!("{} --toggle", canonical.display());
    }

    "whis-desktop --toggle".to_string()
}

#[tauri::command]
pub fn can_reopen_window(state: State<'_, AppState>) -> bool {
    if *state.tray_available.lock().unwrap() {
        return true;
    }
    let backend_info = crate::shortcuts::backend_info();
    match backend_info.backend.as_str() {
        "TauriPlugin" => true,
        "ManualSetup" => true,
        "PortalGlobalShortcuts" => {
            let has_shortcut = state.portal_shortcut.lock().unwrap().is_some();
            let no_error = state.portal_bind_error.lock().unwrap().is_none();
            has_shortcut && no_error
        }
        _ => false,
    }
}

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<whis_core::AudioDeviceInfo>, String> {
    whis_core::list_audio_devices().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn exit_app(app: AppHandle) {
    app.exit(0);
}

/// Get the status of autotyping tools on the system.
#[tauri::command]
pub fn get_autotype_tool_status_cmd() -> AutotypeToolStatus {
    get_autotype_tool_status()
}
