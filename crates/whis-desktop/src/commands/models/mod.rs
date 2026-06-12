//! Model Management Commands (parakeet only)

pub mod downloads;

#[cfg(feature = "local-transcription")]
pub mod parakeet;

#[cfg(feature = "local-transcription")]
pub use parakeet::*;
