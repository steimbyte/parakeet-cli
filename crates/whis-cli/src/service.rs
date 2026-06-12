//! Background service for `whis start` mode

use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::app::TranscriptionConfig;
use crate::hotkey::HotkeyEvent;
use crate::ipc::{IpcMessage, IpcResponse, IpcServer};
use whis_core::{
    AudioRecorder, OutputMethod, Settings, TranscriptionProvider, autotype_text, copy_to_clipboard,
};

type TaskHandle<T> = Arc<Mutex<Option<tokio::task::JoinHandle<T>>>>;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ServiceState {
    Idle,
    Recording,
    Transcribing,
}

pub struct Service {
    state: Arc<Mutex<ServiceState>>,
    recorder: Arc<Mutex<Option<AudioRecorder>>>,
    chunker_handle: TaskHandle<Result<(), String>>,
    transcription_handle: TaskHandle<Result<String>>,
    provider: TranscriptionProvider,
    model_path: String,
    language: Option<String>,
    recording_counter: Arc<Mutex<u32>>,
    output_method_override: Option<OutputMethod>,
}

impl Service {
    pub fn new(config: TranscriptionConfig, output_method_override: Option<OutputMethod>) -> Result<Self> {
        Ok(Self {
            state: Arc::new(Mutex::new(ServiceState::Idle)),
            recorder: Arc::new(Mutex::new(None)),
            chunker_handle: Arc::new(Mutex::new(None)),
            transcription_handle: Arc::new(Mutex::new(None)),
            provider: config.provider,
            model_path: config.model_path,
            language: config.language,
            recording_counter: Arc::new(Mutex::new(0)),
            output_method_override,
        })
    }

    pub async fn run(
        &self,
        mut hotkey_rx: Option<UnboundedReceiver<HotkeyEvent>>,
        push_to_talk: bool,
    ) -> Result<()> {
        let mut ipc_server = IpcServer::new().context("Failed to create IPC server")?;

        // Configure model caching
        #[cfg(feature = "local-transcription")]
        {
            let settings = whis_core::Settings::load_cli();
            let keep_loaded = settings.ui.model_memory.keep_model_loaded;
            self.provider.set_keep_loaded(keep_loaded);
        }

        loop {
            tokio::select! {
                Some(mut conn) = ipc_server.accept() => {
                    match conn.receive() {
                        Ok(message) => {
                            let response = self.handle_message(message).await;
                            let _ = conn.send(response);
                        }
                        Err(e) => {
                            eprintln!("Error receiving message: {e}");
                            let _ = conn.send(IpcResponse::Error(e.to_string()));
                        }
                    }
                }

                Some(event) = async {
                    match &mut hotkey_rx {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    if push_to_talk {
                        match event {
                            HotkeyEvent::Pressed => self.handle_start().await,
                            HotkeyEvent::Released => self.handle_stop().await,
                        }
                    } else if event == HotkeyEvent::Pressed {
                        self.handle_toggle().await;
                    }
                }
            }
        }
    }

    async fn handle_message(&self, message: IpcMessage) -> IpcResponse {
        match message {
            IpcMessage::Toggle => self.handle_toggle().await,
            IpcMessage::Stop => {
                println!("Stop signal received");
                tokio::spawn(async {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    std::process::exit(0);
                });
                IpcResponse::Success
            }
            IpcMessage::Status => {
                let state = *self.state.lock().unwrap();
                match state {
                    ServiceState::Idle => IpcResponse::Idle,
                    ServiceState::Recording => IpcResponse::Recording,
                    ServiceState::Transcribing => IpcResponse::Transcribing,
                }
            }
        }
    }

    async fn handle_toggle(&self) -> IpcResponse {
        let current_state = *self.state.lock().unwrap();
        match current_state {
            ServiceState::Idle => {
                let count = {
                    let mut c = self.recording_counter.lock().unwrap();
                    *c += 1;
                    *c
                };
                match self.start_recording().await {
                    Ok(_) => {
                        println!("#{count} Recording...");
                        IpcResponse::Recording
                    }
                    Err(e) => {
                        println!("#{count} error: {e}");
                        IpcResponse::Error(e.to_string())
                    }
                }
            }
            ServiceState::Recording => {
                *self.state.lock().unwrap() = ServiceState::Transcribing;
                let count = *self.recording_counter.lock().unwrap();
                println!("#{count} Transcribing...");
                match self.stop_and_transcribe(count).await {
                    Ok(_) => {
                        *self.state.lock().unwrap() = ServiceState::Idle;
                        println!();
                        IpcResponse::Success
                    }
                    Err(e) => {
                        *self.state.lock().unwrap() = ServiceState::Idle;
                        println!("#{count} error: {e}");
                        println!();
                        IpcResponse::Error(e.to_string())
                    }
                }
            }
            ServiceState::Transcribing => IpcResponse::Transcribing,
        }
    }

    async fn handle_start(&self) {
        let current_state = *self.state.lock().unwrap();
        if current_state != ServiceState::Idle {
            return;
        }
        let count = {
            let mut c = self.recording_counter.lock().unwrap();
            *c += 1;
            *c
        };
        match self.start_recording().await {
            Ok(_) => println!("#{count} Recording..."),
            Err(e) => println!("#{count} error: {e}"),
        }
    }

    async fn handle_stop(&self) {
        let current_state = *self.state.lock().unwrap();
        if current_state != ServiceState::Recording {
            return;
        }
        *self.state.lock().unwrap() = ServiceState::Transcribing;
        let count = *self.recording_counter.lock().unwrap();
        println!("#{count} Transcribing...");
        match self.stop_and_transcribe(count).await {
            Ok(_) => {
                *self.state.lock().unwrap() = ServiceState::Idle;
                println!();
            }
            Err(e) => {
                *self.state.lock().unwrap() = ServiceState::Idle;
                println!("#{count} error: {e}");
                println!();
            }
        }
    }

    async fn start_recording(&self) -> Result<()> {
        use tokio::sync::mpsc;
        use whis_core::{ChunkerConfig, ProgressiveChunker};

        let mut recorder = AudioRecorder::new()?;
        let settings = Settings::load_cli();
        #[cfg(feature = "vad")]
        {
            recorder.set_vad(settings.ui.vad.enabled, settings.ui.vad.threshold);
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

        let vad_enabled = settings.ui.vad.enabled;
        let target = settings.ui.chunk_duration_secs;
        let chunker_config = ChunkerConfig {
            target_duration_secs: target,
            min_duration_secs: target * 2 / 3,
            max_duration_secs: target * 4 / 3,
            vad_aware: vad_enabled,
        };

        let mut chunker = ProgressiveChunker::new(chunker_config, chunk_tx);
        let chunker_handle = tokio::spawn(async move {
            chunker
                .consume_stream(audio_rx_unbounded, None)
                .await
                .map_err(|e| e.to_string())
        });

        // Local parakeet progressive transcription
        #[cfg(feature = "local-transcription")]
        {
            if let Some(model_path) = settings.transcription.parakeet_model_path() {
                whis_core::preload_parakeet(&model_path);
            }
        }

        let model_path = self.model_path.clone();
        let transcription_handle = tokio::spawn(async move {
            whis_core::progressive_transcribe_local(&model_path, chunk_rx, None).await
        });

        *self.recorder.lock().unwrap() = Some(recorder);
        *self.chunker_handle.lock().unwrap() = Some(chunker_handle);
        *self.transcription_handle.lock().unwrap() = Some(transcription_handle);
        *self.state.lock().unwrap() = ServiceState::Recording;

        Ok(())
    }

    async fn stop_and_transcribe(&self, count: u32) -> Result<()> {
        let mut recorder = self
            .recorder
            .lock()
            .unwrap()
            .take()
            .context("No active recording")?;
        recorder.stop_recording()?;

        let chunker_handle = self
            .chunker_handle
            .lock()
            .unwrap()
            .take()
            .context("No chunker task running")?;
        let transcription_handle = self
            .transcription_handle
            .lock()
            .unwrap()
            .take()
            .context("No transcription task running")?;

        chunker_handle
            .await
            .context("Failed to join chunker task")?
            .map_err(|e| anyhow::anyhow!("Chunker task failed: {}", e))?;

        let text = transcription_handle
            .await
            .context("Failed to join transcription task")??;

        println!("#{count} Done.");

        let settings = Settings::load_cli();
        let clipboard_method = settings.ui.clipboard_backend.clone();
        let output_method = self
            .output_method_override
            .clone()
            .unwrap_or(settings.ui.output_method.clone());
        let autotype_backend = settings.ui.autotype_backend.clone();
        let autotype_delay_ms = settings.ui.autotype_delay_ms;

        tokio::task::spawn_blocking(move || {
            match output_method {
                OutputMethod::Clipboard => {
                    copy_to_clipboard(&text, clipboard_method)?;
                }
                OutputMethod::Autotype => {
                    autotype_text(&text, autotype_backend, autotype_delay_ms)?;
                }
                OutputMethod::Both => {
                    copy_to_clipboard(&text, clipboard_method)?;
                    autotype_text(&text, autotype_backend, autotype_delay_ms)?;
                }
            }
            Ok::<(), anyhow::Error>(())
        })
        .await
        .context("Failed to join task")??;

        Ok(())
    }
}
