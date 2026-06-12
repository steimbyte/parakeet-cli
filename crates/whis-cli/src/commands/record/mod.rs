//! Record Command - Voice-to-Text Pipeline
//!
//! Microphone input uses progressive local Parakeet transcription.
//!
//! ```text
//! ┌─────────────┐      ┌──────────────┐      ┌────────────┐
//! │   Record    │  →   │  Progressive │  →   │   Output   │
//! │  (modes/)   │      │   Parakeet   │      │(pipeline/) │
//! └─────────────┘      └──────────────┘      └────────────┘
//!     ↓                       ↓                     ↓
//! Microphone          ~90s chunks            Clipboard/
//!                     16kHz mono              Stdout/File
//! ```

mod modes;
mod pipeline;
mod types;

pub use types::RecordConfig;

use anyhow::Result;

use crate::app;
use crate::commands::record::pipeline::output::{OutputMode, output};

pub fn run(config: RecordConfig) -> Result<()> {
    let quiet = config.is_quiet();
    let runtime = tokio::runtime::Runtime::new()?;

    let transcription_config =
        app::load_transcription_config_with_language(config.language.clone())?;

    let transcription_result = if let Some(ref input_file) = config.input_file {
        runtime.block_on(transcribe_file(input_file, &transcription_config, quiet))?
    } else {
        let mic_config = modes::MicrophoneConfig {
            duration: config.duration,
            no_vad: config.no_vad,
            provider: transcription_config.provider.clone(),
        };
        runtime.block_on(progressive_record_and_transcribe(
            mic_config,
            &transcription_config,
            quiet,
        ))?
    };

    if !quiet {
        println!(" Done.");
    }

    let output_mode = if config.print {
        OutputMode::Print
    } else if let Some(path) = config.output_path {
        OutputMode::File(path)
    } else {
        OutputMode::Clipboard
    };
    output(transcription_result, output_mode, config.format, quiet)?;

    Ok(())
}

async fn progressive_record_and_transcribe(
    mic_config: modes::MicrophoneConfig,
    transcription_config: &app::TranscriptionConfig,
    quiet: bool,
) -> Result<types::TranscriptionResult> {
    use tokio::sync::mpsc;
    use whis_core::{AudioRecorder, ChunkerConfig, ProgressiveChunker, Settings};

    let mut recorder = AudioRecorder::new()?;
    let settings = Settings::load_cli();
    let vad_enabled = settings.ui.vad.enabled && !mic_config.no_vad;
    recorder.set_vad(vad_enabled, settings.ui.vad.threshold);

    #[cfg(feature = "local-transcription")]
    {
        if let Some(model_path) = settings.transcription.parakeet_model_path() {
            whis_core::preload_parakeet(&model_path);
        }
    }

    let device_name = settings.ui.microphone_device.clone();
    let mut audio_rx_bounded =
        recorder.start_recording_streaming_with_device(device_name.as_deref())?;

    let (audio_tx_unbounded, audio_rx_unbounded) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(samples) = audio_rx_bounded.recv().await {
            if audio_tx_unbounded.send(samples).is_err() {
                break;
            }
        }
    });

    let (chunk_tx, chunk_rx) = mpsc::unbounded_channel();
    let target = settings.ui.chunk_duration_secs;
    let chunker_config = ChunkerConfig {
        target_duration_secs: target,
        min_duration_secs: target * 2 / 3,
        max_duration_secs: target * 4 / 3,
        vad_aware: vad_enabled,
    };

    let mut chunker = ProgressiveChunker::new(chunker_config, chunk_tx);
    let chunker_task = tokio::spawn(async move {
        chunker
            .consume_stream(audio_rx_unbounded, None)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    });

    let model_path = transcription_config.model_path.clone();
    let transcription_task = tokio::spawn(async move {
        whis_core::progressive_transcribe_local(&model_path, chunk_rx, None).await
    });

    if let Some(dur) = mic_config.duration {
        if !quiet {
            if whis_core::verbose::is_verbose() {
                println!("Recording for {} seconds...", dur.as_secs());
            } else {
                print!("Recording for {} seconds...", dur.as_secs());
                use std::io::Write;
                std::io::stdout().flush()?;
            }
        }
        tokio::time::sleep(dur).await;
    } else {
        if !quiet {
            println!("Press Enter to stop");
            if whis_core::verbose::is_verbose() {
                println!("Recording...");
            } else {
                print!("Recording...");
                use std::io::Write;
                std::io::stdout().flush()?;
            }
        }
        tokio::task::spawn_blocking(app::wait_for_stop).await??;
    }

    recorder.stop_recording()?;
    chunker_task.await??;

    if !quiet {
        app::print_status(" Transcribing...", Some(&transcription_config.provider));
    }

    let text = transcription_task.await??;

    Ok(types::TranscriptionResult { text })
}

async fn transcribe_file(
    input_file: &std::path::Path,
    transcription_config: &app::TranscriptionConfig,
    quiet: bool,
) -> Result<types::TranscriptionResult> {
    if !quiet {
        eprintln!(
            "Transcribing {}...",
            input_file.file_name().unwrap_or_default().to_string_lossy()
        );
    }

    let samples = modes::file::read_audio_file(input_file)?;
    let model_path = transcription_config.model_path.clone();

    let text = tokio::task::spawn_blocking(move || {
        whis_core::provider::transcribe_raw_parakeet(&model_path, samples)
    })
    .await??
    .text;

    if !quiet {
        eprintln!("Done.");
    }

    Ok(types::TranscriptionResult { text })
}
