//! External services settings stub.
//!
//! parakeet-cli has no cloud services. This struct is preserved as an
//! empty placeholder so existing settings.json files still deserialize.

use serde::{Deserialize, Serialize};

/// Ollama configuration (stub - parakeet-cli has none).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OllamaConfig {}

/// External services settings (stub - parakeet-cli has none).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServicesSettings {
    /// Ollama configuration (unused in parakeet-cli).
    #[serde(default)]
    pub ollama: OllamaConfig,
}
