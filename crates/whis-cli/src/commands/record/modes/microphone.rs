//! Microphone recording configuration

use std::time::Duration;
use whis_core::TranscriptionProvider;

#[derive(Debug, Clone)]
pub struct MicrophoneConfig {
    pub duration: Option<Duration>,
    pub no_vad: bool,
    pub provider: TranscriptionProvider,
}
