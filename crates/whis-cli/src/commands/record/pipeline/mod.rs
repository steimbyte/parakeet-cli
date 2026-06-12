//! Pipeline phases for the record command
//!
//! parakeet-cli has no post-processing phase. The pipeline is:
//! 1. Record/Load - Get audio from source
//! 2. Progressive Transcribe - parakeet local inference
//! 3. Output - Display, write, or clipboard

pub mod output;
