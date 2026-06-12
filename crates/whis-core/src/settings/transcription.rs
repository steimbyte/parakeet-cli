//! Transcription settings for parakeet-cli (parakeet only).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::TranscriptionProvider;

#[cfg(feature = "local-transcription")]
use crate::model::{ModelType, ParakeetModel};

/// Settings for transcription (parakeet only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSettings {
    /// Active transcription provider (only LocalParakeet in parakeet-cli)
    #[serde(default)]
    pub provider: TranscriptionProvider,

    /// Language hint for transcription (ISO-639-1 code, e.g., "en", "de", "fr")
    /// None = auto-detect
    #[serde(default)]
    pub language: Option<String>,

    /// API keys stored by provider name. Field preserved for settings.json
    /// backwards compatibility; parakeet-cli does not use it.
    #[serde(default)]
    pub api_keys: HashMap<String, String>,

    /// Local model configuration
    #[serde(default)]
    pub local_models: LocalModelsConfig,
}

impl Default for TranscriptionSettings {
    fn default() -> Self {
        Self {
            provider: crate::configuration::DEFAULT_PROVIDER,
            language: crate::configuration::DEFAULT_LANGUAGE.map(String::from),
            api_keys: HashMap::new(),
            local_models: LocalModelsConfig::default(),
        }
    }
}

/// Configuration for local transcription models.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalModelsConfig {
    /// Path to Parakeet model directory
    /// (e.g., ~/.local/share/parakeet-cli/models/parakeet-tdt-0.6b-v3-int8)
    #[serde(default)]
    pub parakeet_path: Option<String>,

    /// Legacy whisper path field - preserved for settings.json backwards compat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whisper_path: Option<String>,
}

impl TranscriptionSettings {
    /// Get the API key for the current provider. parakeet-cli has no API keys.
    pub fn api_key(&self) -> Option<String> {
        None
    }

    /// Get the API key for a specific provider. parakeet-cli has no API keys.
    pub fn api_key_for(&self, _provider: &TranscriptionProvider) -> Option<String> {
        None
    }

    /// Get the API key for the current provider from settings only. parakeet-cli has none.
    pub fn api_key_from_settings(&self) -> Option<String> {
        None
    }

    /// Get the API key for a specific provider from settings only. parakeet-cli has none.
    pub fn api_key_from_settings_for(&self, _provider: &TranscriptionProvider) -> Option<String> {
        None
    }

    /// Check if an API key is explicitly configured. parakeet-cli has none.
    pub fn has_configured_api_key(&self, _provider: &TranscriptionProvider) -> bool {
        false
    }

    /// Set the API key for a provider. parakeet-cli has none, no-op.
    pub fn set_api_key(&mut self, _provider: &TranscriptionProvider, _key: String) {}

    /// Check if an API key is configured for the current provider. parakeet-cli has none.
    pub fn has_api_key(&self) -> bool {
        false
    }

    /// Check if the current provider is properly configured.
    pub fn is_configured(&self) -> bool {
        match self.provider {
            #[cfg(feature = "local-transcription")]
            TranscriptionProvider::LocalParakeet => self
                .parakeet_model_path()
                .map(|p| ParakeetModel.verify(std::path::Path::new(&p)))
                .unwrap_or(false),
            #[cfg(not(feature = "local-transcription"))]
            TranscriptionProvider::LocalParakeet => false,
        }
    }

    /// Get the Parakeet model path, falling back to environment variable.
    pub fn parakeet_model_path(&self) -> Option<String> {
        self.local_models
            .parakeet_path
            .clone()
            .or_else(|| std::env::var("LOCAL_PARAKEET_MODEL_PATH").ok())
    }

    /// Validate transcription settings.
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.is_configured() {
            anyhow::bail!(
                "Parakeet model is not configured. Set it with: \
                 parakeet-cli config --parakeet-model-path <path> \
                 or export LOCAL_PARAKEET_MODEL_PATH."
            );
        }
        Ok(())
    }
}
