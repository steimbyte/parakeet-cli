//! Application Settings Module
//!
//! parakeet-cli: only the local Parakeet provider is supported. The
//! `post_processing` and `services` fields exist as stubs for settings.json
//! backwards compatibility.

mod post_processing;
mod services;
mod shortcuts;
mod transcription;
mod ui;

pub use post_processing::PostProcessingSettings;
pub use services::{OllamaConfig, ServicesSettings};
pub use shortcuts::{CliShortcutMode, ShortcutsSettings};
pub use transcription::{LocalModelsConfig, TranscriptionSettings};
pub use ui::{BubbleSettings, ModelMemorySettings, UiSettings, VadSettings};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application settings (aggregate root).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub transcription: TranscriptionSettings,
    pub post_processing: PostProcessingSettings,
    pub services: ServicesSettings,
    pub shortcuts: ShortcutsSettings,
    pub ui: UiSettings,
}

impl Settings {
    /// Get the CLI settings file path (~/.config/parakeet-cli/settings.json).
    pub fn cli_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("parakeet-cli")
            .join("settings.json")
    }

    /// Load settings from CLI config file.
    pub fn load_cli() -> Self {
        let path = Self::cli_path();
        if let Ok(content) = fs::read_to_string(&path) {
            match serde_json::from_str(&content) {
                Ok(settings) => return settings,
                Err(e) => {
                    eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                }
            }
        }
        Self::default()
    }

    /// Save settings to CLI config file with 0600 permissions.
    pub fn save_cli(&self) -> Result<()> {
        use std::io::Write;

        let path = Self::cli_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)?;
            file.write_all(content.as_bytes())?;
        }

        #[cfg(not(unix))]
        {
            fs::write(&path, &content)?;
        }

        Ok(())
    }

    /// Validate all settings.
    pub fn validate(&self) -> Result<()> {
        self.transcription.validate()?;
        self.shortcuts.validate()?;
        Ok(())
    }
}
