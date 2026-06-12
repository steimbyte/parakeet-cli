//! Tauri Command Handlers (parakeet-only)

use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use whis_core::Settings;

/// Save settings to Tauri store (shared helper for all commands)
pub(crate) fn save_settings_to_store(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to open store: {e}"))?;
    store.set(
        "settings",
        serde_json::to_value(settings).map_err(|e| format!("Failed to serialize settings: {e}"))?,
    );
    store
        .save()
        .map_err(|e| format!("Failed to save settings: {e}"))
}

pub mod bubble;
pub mod models;
pub mod recording;
pub mod settings;
pub mod shortcuts;
pub mod system;
pub mod validation;

pub use system::*;
pub use validation::*;
pub use recording::*;
pub use settings::*;
pub use shortcuts::*;
pub use models::*;
pub use bubble::*;
