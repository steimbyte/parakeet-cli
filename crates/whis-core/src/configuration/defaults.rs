//! Defaults for parakeet-cli

use crate::TranscriptionProvider;

/// Default transcription provider: local Parakeet
pub const DEFAULT_PROVIDER: TranscriptionProvider = TranscriptionProvider::LocalParakeet;

/// Default language (None = auto-detect)
pub const DEFAULT_LANGUAGE: Option<&str> = None;

// =============================================================================
// UI DEFAULTS
// =============================================================================

/// Default shortcut mode for triggering recording
pub const DEFAULT_SHORTCUT_MODE: &str = "system";

/// Default keyboard shortcut for recording toggle
pub const DEFAULT_SHORTCUT: &str = "Ctrl+Alt+P";

/// Default VAD enabled state
pub const DEFAULT_VAD_ENABLED: bool = false;

/// Default VAD threshold (0.0 = silence, 1.0 = loud speech)
pub const DEFAULT_VAD_THRESHOLD: f32 = 0.5;

/// Default chunk duration for progressive transcription (seconds)
pub const DEFAULT_CHUNK_DURATION_SECS: u64 = 90;

// =============================================================================
// MODEL MEMORY DEFAULTS
// =============================================================================

/// Whether to keep the parakeet model loaded in memory between transcriptions
pub const DEFAULT_KEEP_MODEL_LOADED: bool = true;

/// Auto-unload timeout in minutes (0 = never auto-unload)
pub const DEFAULT_MODEL_UNLOAD_MINUTES: u32 = 10;
