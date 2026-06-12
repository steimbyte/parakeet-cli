//! parakeet-cli core library
//!
//! Local Parakeet ASR only. No cloud providers, no LLM post-processing.

pub mod audio;
pub mod configuration;
pub mod model;
pub mod provider;
pub mod settings;
pub mod transcription;

#[cfg(feature = "autotyping")]
pub mod autotyping;
#[cfg(feature = "clipboard")]
pub mod clipboard;
pub mod error;
#[cfg(feature = "hotkey")]
pub mod hotkey;
pub mod platform;
pub mod resample;
pub mod state;
pub mod verbose;

// Re-export audio types
pub use audio::{
    AudioDeviceInfo, AudioRecorder, ChunkerConfig, ProgressiveChunk, ProgressiveChunker,
    RecordingData, VadConfig, list_audio_devices,
};

// Re-export configuration types
pub use configuration::{
    DEFAULT_CHUNK_DURATION_SECS, DEFAULT_KEEP_MODEL_LOADED, DEFAULT_LANGUAGE,
    DEFAULT_MODEL_UNLOAD_MINUTES, DEFAULT_PROVIDER, DEFAULT_SHORTCUT, DEFAULT_SHORTCUT_MODE,
    DEFAULT_VAD_ENABLED, DEFAULT_VAD_THRESHOLD,
};
pub use configuration::{Preset, PresetSource, TranscriptionProvider};

// Re-export transcription
pub use transcription::progressive_transcribe_local;

// Re-export provider types
#[cfg(feature = "local-transcription")]
pub use provider::preload_parakeet;
#[cfg(feature = "local-transcription")]
pub use provider::transcribe_raw_parakeet;
#[cfg(feature = "local-transcription")]
pub use provider::transcribe_raw_parakeet as transcribe_raw;
pub use provider::{
    DEFAULT_TIMEOUT_SECS, ProgressCallback, TranscriptionBackend, TranscriptionRequest,
    TranscriptionResult, TranscriptionStage, is_realtime_provider, registry,
};
#[cfg(feature = "local-transcription")]
pub use provider::{parakeet_set_keep_loaded, unload_parakeet};

// Re-export other utility types
#[cfg(feature = "autotyping")]
pub use autotyping::{
    AutotypeBackend, AutotypeToolStatus, OutputMethod, autotype_text, get_autotype_tool_status,
};
#[cfg(feature = "clipboard")]
pub use clipboard::{ClipboardMethod, copy_to_clipboard};
pub use error::{AudioError, ProviderError, Result, WhisError};
pub use settings::Settings;
pub use state::RecordingState;
pub use verbose::set_verbose;

#[cfg(feature = "hotkey")]
pub use hotkey::{Hotkey, HotkeyParseError, key_to_string, lock_or_recover, parse_key};
pub use platform::{Compositor, Platform, PlatformInfo, detect_platform, is_flatpak};

// Legacy module aliases for backward compatibility
#[doc(hidden)]
pub mod config {
    pub use crate::configuration::TranscriptionProvider;
}

#[doc(hidden)]
pub mod defaults {
    pub use crate::configuration::{
        DEFAULT_KEEP_MODEL_LOADED, DEFAULT_LANGUAGE, DEFAULT_MODEL_UNLOAD_MINUTES,
        DEFAULT_PROVIDER, DEFAULT_SHORTCUT, DEFAULT_SHORTCUT_MODE, DEFAULT_VAD_ENABLED,
        DEFAULT_VAD_THRESHOLD,
    };
}

#[doc(hidden)]
pub mod preset {
    pub use crate::configuration::{Preset, PresetSource};
}

#[doc(hidden)]
pub mod transcribe {
    pub use crate::transcription::progressive_transcribe_local;
}
