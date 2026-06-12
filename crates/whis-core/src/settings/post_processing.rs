//! Post-processing settings stub.
//!
//! parakeet-cli has no LLM post-processing. This struct is preserved as an
//! empty placeholder so existing settings.json files still deserialize.

use serde::{Deserialize, Serialize};

/// Settings for post-processing (stub - parakeet-cli has none).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostProcessingSettings {
    /// Whether post-processing is enabled. Always false in parakeet-cli.
    #[serde(default)]
    pub enabled: bool,
}

impl PostProcessingSettings {
    /// Get the API key for the post-processor. None - parakeet-cli has none.
    pub fn api_key(
        &self,
        _transcription_api_keys: &std::collections::HashMap<String, String>,
    ) -> Option<String> {
        None
    }

    /// Get the API key for the post-processor from settings only. None.
    pub fn api_key_from_settings(
        &self,
        _transcription_api_keys: &std::collections::HashMap<String, String>,
    ) -> Option<String> {
        None
    }

    /// Check if post-processing is enabled and properly configured. Always false.
    pub fn is_configured(
        &self,
        _transcription_api_keys: &std::collections::HashMap<String, String>,
    ) -> bool {
        false
    }

    /// Validate post-processing settings. Always succeeds.
    pub fn validate(
        &self,
        _transcription_api_keys: &std::collections::HashMap<String, String>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
