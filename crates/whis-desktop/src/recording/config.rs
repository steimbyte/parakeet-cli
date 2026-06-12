//! Recording Configuration (parakeet only)

use crate::state::{AppState, TranscriptionConfig};

/// Load transcription configuration from settings
pub fn load_transcription_config(state: &AppState) -> Result<TranscriptionConfig, String> {
    let settings = state.settings.lock().unwrap();
    let provider = settings.transcription.provider.clone();
    let model_path = settings
        .transcription
        .parakeet_model_path()
        .ok_or_else(|| "Parakeet model not configured. Add it in Settings.".to_string())?;
    let language = settings.transcription.language.clone();
    Ok(TranscriptionConfig {
        provider,
        model_path,
        language,
    })
}
