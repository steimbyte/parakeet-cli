//! Recording Control
//!
//! Starts/stops audio recording, runs progressive local Parakeet transcription.

use super::config::load_transcription_config;
use crate::state::{AppState, RecordingState};
use tauri::AppHandle;
use tokio::sync::{mpsc, oneshot};
use whis_core::{
    AudioRecorder, ChunkerConfig, ProgressiveChunker, info, progressive_transcribe_local,
};

/// Start recording with progressive Parakeet transcription
pub fn start_recording_sync(_app: &AppHandle, state: &AppState) -> Result<(), String> {
    state.cancel_idle_unload();

    let (parakeet_model_path, language) = {
        let mut config_guard = state.transcription_config.lock().unwrap();
        if config_guard.is_none() {
            *config_guard = Some(load_transcription_config(state)?);
        }
        let config = config_guard.as_ref().unwrap();
        (config.model_path.clone(), config.language.clone())
    };

    let settings = state.settings.lock().unwrap();
    let vad_enabled = settings.ui.vad.enabled;
    let vad_threshold = settings.ui.vad.threshold;
    let device_name = settings.ui.microphone_device.clone();
    let chunk_duration = settings.ui.chunk_duration_secs;
    let keep_loaded = settings.ui.model_memory.keep_model_loaded;
    drop(settings);

    let mut recorder = AudioRecorder::new().map_err(|e| e.to_string())?;
    recorder.set_vad(vad_enabled, vad_threshold);

    let mut audio_rx_bounded = recorder
        .start_recording_streaming_with_device(device_name.as_deref())
        .map_err(|e| e.to_string())?;

    let (audio_tx_unbounded, audio_rx_unbounded) = mpsc::unbounded_channel();
    tauri::async_runtime::spawn(async move {
        while let Some(samples) = audio_rx_bounded.recv().await {
            if audio_tx_unbounded.send(samples).is_err() {
                break;
            }
        }
    });

    let (result_tx, result_rx) = oneshot::channel();

    // Preload parakeet model in background
    whis_core::preload_parakeet(&parakeet_model_path);

    // Set keep-loaded for the model memory
    let provider = whis_core::TranscriptionProvider::LocalParakeet;
    provider.set_keep_loaded(keep_loaded);

    let (chunk_tx, chunk_rx) = mpsc::unbounded_channel();
    let chunker_config = ChunkerConfig {
        target_duration_secs: chunk_duration,
        min_duration_secs: chunk_duration * 2 / 3,
        max_duration_secs: chunk_duration * 4 / 3,
        vad_aware: vad_enabled,
    };

    let mut chunker = ProgressiveChunker::new(chunker_config, chunk_tx);
    tauri::async_runtime::spawn(async move {
        let _ = chunker.consume_stream(audio_rx_unbounded, None).await;
    });

    let model_path_for_tx = parakeet_model_path.clone();
    tauri::async_runtime::spawn(async move {
        let result: Result<String, String> = progressive_transcribe_local(
            &model_path_for_tx,
            chunk_rx,
            None,
        )
        .await
        .map_err(|e| e.to_string());
        let _ = result_tx.send(result);
    });

    info!("Recording started (parakeet local progressive mode)");

    *state.transcription_rx.lock().unwrap() = Some(result_rx);
    *state.recorder.lock().unwrap() = Some(recorder);
    *state.state.lock().unwrap() = RecordingState::Recording;

    Ok(())
}
