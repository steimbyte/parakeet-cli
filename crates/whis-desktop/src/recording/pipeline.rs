//! Transcription Pipeline (parakeet local only)
//!
//! 1. Stop recording (finalize audio)
//! 2. Transcribe audio (parakeet, progressive)
//! 3. Copy to clipboard / autotype
//! 4. Emit completion event

use crate::state::{AppState, RecordingState};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use whis_core::{
    AutotypeBackend, ClipboardMethod, OutputMethod, TranscriptionProvider, autotype_text,
    copy_to_clipboard,
};

#[cfg(feature = "local-transcription")]
use whis_core::unload_parakeet;

fn output_text(
    text: &str,
    output_method: &OutputMethod,
    clipboard_method: &ClipboardMethod,
    autotype_backend: &AutotypeBackend,
    autotype_delay_ms: Option<u32>,
) -> Result<(), String> {
    match output_method {
        OutputMethod::Clipboard => {
            copy_to_clipboard(text, clipboard_method.clone()).map_err(|e| e.to_string())?;
        }
        OutputMethod::Autotype => {
            autotype_text(text, autotype_backend.clone(), autotype_delay_ms)
                .map_err(|e| e.to_string())?;
        }
        OutputMethod::Both => {
            copy_to_clipboard(text, clipboard_method.clone()).map_err(|e| e.to_string())?;
            autotype_text(text, autotype_backend.clone(), autotype_delay_ms)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub async fn stop_and_transcribe(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();

    {
        let mut recorder = state.recorder.lock().unwrap().take();
        if let Some(ref mut rec) = recorder {
            rec.stop_recording().map_err(|e| e.to_string())?;
        }
    }
    {
        *state.state.lock().unwrap() = RecordingState::Transcribing;
    }
    println!("Transcribing...");

    let result = do_progressive_transcription(app, &state).await;

    {
        *state.state.lock().unwrap() = RecordingState::Idle;
    }

    result
}

async fn do_progressive_transcription(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let rx = {
        let mut rx_guard = state.transcription_rx.lock().unwrap();
        rx_guard
            .take()
            .ok_or("No progressive transcription in progress")?
    };

    let transcription = rx
        .await
        .map_err(|_| "Transcription task dropped unexpectedly".to_string())?
        .map_err(|e| format!("Transcription failed: {e}"))?;

    let (clipboard_method, output_method, autotype_backend, autotype_delay_ms) = {
        let settings = state.settings.lock().unwrap();
        (
            settings.ui.clipboard_backend.clone(),
            settings.ui.output_method.clone(),
            settings.ui.autotype_backend.clone(),
            settings.ui.autotype_delay_ms,
        )
    };

    output_text(
        &transcription,
        &output_method,
        &clipboard_method,
        &autotype_backend,
        autotype_delay_ms,
    )?;

    println!("Done: {}", &transcription[..transcription.len().min(50)]);

    let _ = app.emit("transcription-complete", &transcription);

    schedule_idle_model_unload(app, state);
    Ok(())
}

fn schedule_idle_model_unload(app: &AppHandle, state: &AppState) {
    let (keep_loaded, unload_minutes) = {
        let settings = state.settings.lock().unwrap();
        (
            settings.ui.model_memory.keep_model_loaded,
            settings.ui.model_memory.unload_after_minutes,
        )
    };

    #[cfg(feature = "local-transcription")]
    if keep_loaded && unload_minutes > 0 {
        let duration = Duration::from_secs(u64::from(unload_minutes) * 60);
        let app_handle = app.clone();

        let task = tauri::async_runtime::spawn(async move {
            tokio::time::sleep(duration).await;

            let state = app_handle.state::<AppState>();
            let should_unload = {
                let settings = state.settings.lock().unwrap();
                settings.ui.model_memory.keep_model_loaded
                    && settings.ui.model_memory.unload_after_minutes > 0
            };

            if should_unload {
                println!(
                    "Auto-unloading parakeet model after {} minutes of inactivity",
                    unload_minutes
                );
                unload_parakeet();
            }
        });

        state.set_idle_unload_handle(task);
    }

    #[cfg(not(feature = "local-transcription"))]
    {
        let _ = (keep_loaded, unload_minutes, app);
    }
}
