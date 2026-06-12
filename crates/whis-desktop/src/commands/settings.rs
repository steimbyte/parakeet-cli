//! Settings Management Commands (parakeet-only)

use super::save_settings_to_store;
use crate::state::AppState;
use tauri::{AppHandle, State};
use whis_core::{
    Settings,
    model::{ModelType, ParakeetModel},
};

#[derive(serde::Serialize)]
pub struct SaveSettingsResponse {
    pub needs_restart: bool,
}

#[derive(serde::Serialize)]
pub struct ConfigReadiness {
    pub transcription_ready: bool,
    pub transcription_error: Option<String>,
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let settings = state.settings.lock().unwrap();
    Ok(settings.clone())
}

#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<SaveSettingsResponse, String> {
    let (config_changed, shortcut_changed) = {
        let current = state.settings.lock().unwrap();
        (
            current.transcription.provider != settings.transcription.provider
                || current.transcription.language != settings.transcription.language
                || current.transcription.local_models.parakeet_path
                    != settings.transcription.local_models.parakeet_path,
            current.shortcuts.desktop_key != settings.shortcuts.desktop_key,
        )
    };

    {
        let mut state_settings = state.settings.lock().unwrap();
        *state_settings = settings.clone();
    }

    save_settings_to_store(&app, &settings)?;

    if config_changed {
        *state.transcription_config.lock().unwrap() = None;
    }

    let needs_restart = if shortcut_changed {
        crate::shortcuts::update_shortcut(&app, &settings.shortcuts.desktop_key)
            .map_err(|e| e.to_string())?
    } else {
        false
    };

    Ok(SaveSettingsResponse { needs_restart })
}

/// Check if the parakeet model is properly configured
#[tauri::command]
pub async fn check_config_readiness(
    parakeet_model_path: Option<String>,
) -> ConfigReadiness {
    let (transcription_ready, transcription_error) = match &parakeet_model_path {
        Some(path) if ParakeetModel.verify(std::path::Path::new(path)) => (true, None),
        Some(_) => (false, Some("Parakeet model not found or invalid".to_string())),
        None => (false, Some("Parakeet model not configured".to_string())),
    };

    ConfigReadiness {
        transcription_ready,
        transcription_error,
    }
}

/// Get canonical default values from whis-core
#[tauri::command]
pub fn get_defaults() -> serde_json::Value {
    use whis_core::defaults::*;

    serde_json::json!({
        "provider": DEFAULT_PROVIDER.as_str(),
        "desktop_key": DEFAULT_SHORTCUT,
        "vad_enabled": DEFAULT_VAD_ENABLED,
        "vad_threshold": DEFAULT_VAD_THRESHOLD,
    })
}
