//! Transcription pipeline for parakeet (local only).
//!
//! No cloud fallbacks, no LLM post-processing — just local parakeet.

mod transcribe;

pub use transcribe::progressive_transcribe_local;
