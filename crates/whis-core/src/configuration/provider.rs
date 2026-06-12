//! Transcription provider configuration.
//!
//! Only the local Parakeet provider is supported in parakeet-cli.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Available transcription providers (parakeet only)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptionProvider {
    #[serde(rename = "local-parakeet")]
    LocalParakeet,
}

impl Default for TranscriptionProvider {
    fn default() -> Self {
        super::DEFAULT_PROVIDER
    }
}

impl TranscriptionProvider {
    /// Get the string identifier for this provider
    pub fn as_str(&self) -> &'static str {
        match self {
            TranscriptionProvider::LocalParakeet => "local-parakeet",
        }
    }

    /// Get the environment variable name for this provider's model path
    pub fn api_key_env_var(&self) -> &'static str {
        match self {
            TranscriptionProvider::LocalParakeet => "LOCAL_PARAKEET_MODEL_PATH",
        }
    }

    /// List all available providers.
    pub fn all() -> &'static [TranscriptionProvider] {
        &[TranscriptionProvider::LocalParakeet]
    }

    /// Human-readable display name for this provider
    pub fn display_name(&self) -> &'static str {
        match self {
            TranscriptionProvider::LocalParakeet => "Local Parakeet",
        }
    }

    /// Whether this provider requires an API key (parakeet: no, uses model path)
    pub fn requires_api_key(&self) -> bool {
        false
    }

    /// Whether this is a local provider (yes, always local in parakeet-cli)
    pub fn is_local(&self) -> bool {
        true
    }

    /// Configure model memory behavior for local providers.
    #[cfg(feature = "local-transcription")]
    pub fn set_keep_loaded(&self, keep: bool) {
        match self {
            Self::LocalParakeet => crate::provider::parakeet_set_keep_loaded(keep),
        }
    }
}

impl fmt::Display for TranscriptionProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for TranscriptionProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local-parakeet" | "localparakeet" | "parakeet" => {
                Ok(TranscriptionProvider::LocalParakeet)
            }
            _ => Err(format!(
                "Unknown provider: {}. Only 'local-parakeet' is supported in parakeet-cli.",
                s
            )),
        }
    }
}
