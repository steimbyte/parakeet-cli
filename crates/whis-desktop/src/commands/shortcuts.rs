//! Shortcut Configuration Commands
//!
//! Provides Tauri commands for configuring and managing global keyboard shortcuts.

use crate::shortcuts::ShortcutBackendInfo;
use crate::state::AppState;
use tauri::{AppHandle, State};

/// Get the current shortcut backend information
#[tauri::command]
pub fn shortcut_backend() -> ShortcutBackendInfo {
    crate::shortcuts::backend_info()
}

/// Open shortcut configuration dialog (Portal v2+) or bind directly (Portal v1)
#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn configure_shortcut(app: AppHandle) -> Result<Option<String>, String> {
    crate::shortcuts::open_configure_shortcuts(app)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub async fn configure_shortcut(_app: AppHandle) -> Result<Option<String>, String> {
    Err("Portal shortcuts are only supported on Linux".to_string())
}

/// Configure shortcut with a preferred trigger from in-app key capture
/// The trigger should be in human-readable format like "Ctrl+Alt+W" or "Cmd+Option+W"
#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn configure_shortcut_with_trigger(
    app: AppHandle,
    trigger: String,
) -> Result<Option<String>, String> {
    crate::shortcuts::configure_with_preferred_trigger(Some(&trigger), app)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub async fn configure_shortcut_with_trigger(
    _app: AppHandle,
    _trigger: String,
) -> Result<Option<String>, String> {
    Err("Portal shortcuts are only supported on Linux".to_string())
}

/// Get the currently configured portal shortcut
/// Returns cached value or reads from dconf (GNOME)
#[cfg(target_os = "linux")]
#[tauri::command]
pub fn portal_shortcut(state: State<'_, AppState>) -> Result<Option<String>, String> {
    // First check if we have it cached in state
    let cached = state.portal_shortcut.lock().unwrap().clone();
    if cached.is_some() {
        return Ok(cached);
    }

    // Otherwise try reading from dconf (GNOME stores shortcuts there)
    Ok(crate::shortcuts::read_portal_shortcut_from_dconf())
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub fn portal_shortcut(state: State<'_, AppState>) -> Result<Option<String>, String> {
    // On non-Linux, just return cached value (portal shortcuts are Linux-only)
    Ok(state.portal_shortcut.lock().unwrap().clone())
}

/// Reset portal shortcuts by clearing dconf (GNOME)
/// This allows rebinding after restart
#[cfg(target_os = "linux")]
#[tauri::command]
pub fn reset_shortcut() -> Result<(), String> {
    std::process::Command::new("dconf")
        .args([
            "reset",
            "-f",
            "/org/gnome/settings-daemon/global-shortcuts/",
        ])
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub fn reset_shortcut() -> Result<(), String> {
    Ok(())
}

/// Get any error from portal shortcut binding
#[tauri::command]
pub fn portal_bind_error(state: State<'_, AppState>) -> Option<String> {
    state.portal_bind_error.lock().unwrap().clone()
}

/// Get any error from rdev grab (Linux only)
#[cfg(target_os = "linux")]
#[tauri::command]
pub fn rdev_grab_error(state: State<'_, AppState>) -> Option<String> {
    state.rdev_grab_error.lock().unwrap().clone()
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub fn rdev_grab_error() -> Option<String> {
    None
}

/// Check if current user is in the 'input' group (Linux only)
/// Required for rdev::grab() to work on Wayland
#[tauri::command]
pub fn check_input_group_membership() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("id")
            .args(["-nG"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("input"))
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        true // Non-Linux platforms don't need input group
    }
}

/// Open native keyboard settings app for the given compositor
#[tauri::command]
pub fn open_keyboard_settings(compositor: String) -> Result<(), String> {
    let cmd = match compositor.to_lowercase().as_str() {
        s if s.contains("gnome") => "gnome-control-center keyboard",
        s if s.contains("kde") || s.contains("plasma") => "systemsettings kcm_keys",
        _ => return Err("No settings app available for this compositor".into()),
    };

    std::process::Command::new("sh")
        .args(["-c", cmd])
        .spawn()
        .map_err(|e| format!("Failed to open settings: {e}"))?;

    Ok(())
}

/// Get setup instructions for the current compositor
#[tauri::command]
pub fn get_shortcut_instructions(shortcut: String) -> ShortcutInstructions {
    let capability = crate::shortcuts::detect_backend();
    let compositor = &capability.platform_info.compositor;

    ShortcutInstructions {
        compositor_name: compositor.display_name().to_string(),
        instructions: crate::shortcuts::get_instructions(compositor, &shortcut),
        config_path: crate::shortcuts::get_config_path(compositor).map(|s| s.to_string()),
        config_snippet: crate::shortcuts::get_config_snippet(compositor, &shortcut),
        has_settings_app: matches!(
            compositor,
            whis_core::Compositor::Gnome | whis_core::Compositor::KdePlasma
        ),
    }
}

/// Instructions for setting up shortcuts
#[derive(Clone, serde::Serialize)]
pub struct ShortcutInstructions {
    pub compositor_name: String,
    pub instructions: String,
    pub config_path: Option<String>,
    pub config_snippet: Option<String>,
    pub has_settings_app: bool,
}

/// Get the custom shortcut configured in GNOME Settings (if any)
///
/// Scans GNOME's dconf custom shortcuts for any shortcut that
/// executes `whis-desktop --toggle`. Returns the binding in
/// human-readable format like "Ctrl+Alt+F".
#[cfg(target_os = "linux")]
#[tauri::command]
pub fn system_shortcut_from_dconf() -> Option<String> {
    crate::shortcuts::read_gnome_custom_shortcut()
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub fn system_shortcut_from_dconf() -> Option<String> {
    None
}

/// Information about a shortcut path mismatch
#[derive(Clone, serde::Serialize)]
pub struct ShortcutPathMismatch {
    /// The command configured in dconf (may point to old binary)
    pub configured_command: String,
    /// The expected command for the current binary
    pub current_command: String,
}

/// Check if the configured shortcut command path matches the current binary path
///
/// Returns mismatch information if the configured command in dconf differs
/// from the current executable's toggle command. Returns None if they match
/// or if no shortcut is configured.
#[cfg(target_os = "linux")]
#[tauri::command]
pub fn check_shortcut_path_mismatch() -> Option<ShortcutPathMismatch> {
    let configured_command = crate::shortcuts::read_gnome_custom_shortcut_command()?;
    let current_command = super::system::get_toggle_command();
    let configured_trimmed = configured_command.trim();
    let current_trimmed = current_command.trim();

    if configured_trimmed != current_trimmed {
        Some(ShortcutPathMismatch {
            configured_command,
            current_command,
        })
    } else {
        None
    }
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub fn check_shortcut_path_mismatch() -> Option<ShortcutPathMismatch> {
    None
}

/// Update the GNOME custom shortcut command to use the current binary path
///
/// Finds the custom shortcut for whis and updates its command to the current binary path.
#[cfg(target_os = "linux")]
#[tauri::command]
pub fn update_shortcut_command() -> Result<(), String> {
    use std::process::Command;

    // First, find which custom keybinding contains the whis command
    let output = Command::new("dconf")
        .args([
            "dump",
            "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/",
        ])
        .output()
        .map_err(|e| format!("Failed to read dconf: {e}"))?;

    let dump = String::from_utf8_lossy(&output.stdout);
    let mut current_section: Option<String> = None;

    for line in dump.lines() {
        if line.starts_with('[') && line.ends_with(']') {
            current_section = Some(line[1..line.len() - 1].to_string());
        }

        if line.starts_with("command=") {
            let cmd = line
                .trim_start_matches("command=")
                .trim_matches(|c| c == '\'' || c == '"');
            if cmd.to_lowercase().contains("whis")
                && cmd.contains("--toggle")
                && let Some(section) = &current_section
            {
                // Found the section, update the command
                let new_command = super::system::get_toggle_command();
                let dconf_path = format!(
                    "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/{}/command",
                    section
                );

                Command::new("dconf")
                    .args(["write", &dconf_path, &format!("'{}'", new_command)])
                    .status()
                    .map_err(|e| format!("Failed to write dconf: {e}"))?;

                return Ok(());
            }
        }
    }

    Err("No whis shortcut found in GNOME custom keybindings".to_string())
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
pub fn update_shortcut_command() -> Result<(), String> {
    Err("Not supported on this platform".to_string())
}
