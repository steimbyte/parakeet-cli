//! Model Management Module (parakeet only)

pub mod download;
#[cfg(feature = "local-transcription")]
pub mod parakeet;
pub mod types;

// Re-export commonly used types
pub use types::{ModelInfo, ModelType};
#[cfg(feature = "local-transcription")]
pub use parakeet::ParakeetModel;

#[cfg(feature = "local-transcription")]
pub const DEFAULT_PARAKEET_MODEL: &str = parakeet::DEFAULT_MODEL;
