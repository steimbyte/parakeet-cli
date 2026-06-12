//! Transcription Provider Module (parakeet-only)
//!
//! Only the local Parakeet provider is supported. Cloud providers have been removed.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

/// Stages of the transcription workflow for progress reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptionStage {
    /// Recording audio from microphone
    Recording,
    /// Transcribing audio (local inference)
    Transcribing,
    /// Transcription complete
    Complete,
}

impl TranscriptionStage {
    /// Get a human-readable status message for this stage
    pub fn message(&self) -> &'static str {
        match self {
            Self::Recording => "Recording...",
            Self::Transcribing => "Transcribing...",
            Self::Complete => "Done!",
        }
    }
}

/// Progress callback type for reporting transcription stages
pub type ProgressCallback = Arc<dyn Fn(TranscriptionStage) + Send + Sync>;

mod base;
pub mod error;
#[cfg(feature = "local-transcription")]
mod local_parakeet;

/// Default timeout for API requests (unused for local parakeet, kept for API compat)
pub const DEFAULT_TIMEOUT_SECS: u64 = 300;

pub use error::ProviderError;
#[cfg(feature = "local-transcription")]
pub use local_parakeet::LocalParakeetProvider;
#[cfg(feature = "local-transcription")]
pub use local_parakeet::preload_parakeet;
#[cfg(feature = "local-transcription")]
pub use local_parakeet::transcribe_raw as transcribe_raw_parakeet;
#[cfg(feature = "local-transcription")]
pub use local_parakeet::{set_keep_loaded as parakeet_set_keep_loaded, unload_parakeet};

use crate::config::TranscriptionProvider;

/// Request data for transcription
#[derive(Clone)]
pub struct TranscriptionRequest {
    pub audio_data: Vec<u8>,
    pub language: Option<String>,
    pub filename: String,
    pub mime_type: String,
    /// Optional progress callback for status updates
    pub progress: Option<ProgressCallback>,
}

impl TranscriptionRequest {
    /// Create a new request without progress callback
    pub fn new(audio_data: Vec<u8>, language: Option<String>) -> Self {
        Self {
            audio_data,
            language,
            filename: "audio.mp3".to_string(),
            mime_type: "audio/mpeg".to_string(),
            progress: None,
        }
    }

    /// Set the progress callback
    pub fn with_progress(mut self, callback: ProgressCallback) -> Self {
        self.progress = Some(callback);
        self
    }

    /// Report progress if callback is set
    pub fn report(&self, stage: TranscriptionStage) {
        if let Some(cb) = &self.progress {
            cb(stage);
        }
    }
}

/// Result of a transcription
pub struct TranscriptionResult {
    pub text: String,
}

/// Trait for transcription providers.
///
/// All providers (only Parakeet for now) implement this trait.
#[async_trait]
pub trait TranscriptionBackend: Send + Sync {
    /// Unique identifier for this provider (e.g., "local-parakeet")
    fn name(&self) -> &'static str;

    /// Display name for UI (e.g., "Local Parakeet")
    fn display_name(&self) -> &'static str;

    /// Synchronous transcription (for simple single-file case)
    fn transcribe_sync(
        &self,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult>;

    /// Async transcription for chunk processing
    async fn transcribe_async(
        &self,
        client: &reqwest::Client,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult>;
}

/// Registry of all available transcription providers (parakeet only)
pub struct ProviderRegistry {
    providers: HashMap<&'static str, Arc<dyn TranscriptionBackend>>,
}

impl ProviderRegistry {
    /// Create registry with the parakeet provider
    pub fn new() -> Self {
        let mut providers: HashMap<&'static str, Arc<dyn TranscriptionBackend>> = HashMap::new();

        #[cfg(feature = "local-transcription")]
        providers.insert("local-parakeet", Arc::new(LocalParakeetProvider));

        Self { providers }
    }

    /// Get a provider by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn TranscriptionBackend>> {
        self.providers.get(name).cloned()
    }

    /// List all provider names
    pub fn list(&self) -> Vec<&'static str> {
        self.providers.keys().copied().collect()
    }

    /// Get provider for a TranscriptionProvider enum value
    pub fn get_by_kind(
        &self,
        kind: &TranscriptionProvider,
    ) -> Result<Arc<dyn TranscriptionBackend>> {
        self.get(kind.as_str()).ok_or_else(|| {
            anyhow::anyhow!(
                "Provider '{}' not found in registry. This is a bug.",
                kind.as_str()
            )
        })
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the global provider registry
pub fn registry() -> &'static ProviderRegistry {
    static REGISTRY: OnceLock<ProviderRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ProviderRegistry::new)
}

/// Check if a provider supports realtime WebSocket streaming.
///
/// parakeet-cli has no realtime providers — always returns false.
pub fn is_realtime_provider(_provider: &TranscriptionProvider) -> bool {
    false
}
