use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{IsTerminal, Write};
use std::thread;
use std::time::Duration;
use whis_core::{Settings, TranscriptionProvider};

/// Configuration for transcription
pub struct TranscriptionConfig {
    pub provider: TranscriptionProvider,
    /// Model path (parakeet only)
    pub model_path: String,
    pub language: Option<String>,
}

/// Load transcription config with optional language override
pub fn load_transcription_config_with_language(
    language_override: Option<String>,
) -> Result<TranscriptionConfig> {
    let settings = Settings::load_cli();
    let provider = settings.transcription.provider.clone();
    let language = language_override.or_else(|| settings.transcription.language.clone());

    let model_path = match settings.transcription.parakeet_model_path() {
        Some(path) => path,
        None => {
            eprintln!("Error: No Parakeet model path configured.");
            eprintln!();
            eprintln!("Set the model path with:");
            eprintln!(
                "  whis config parakeet-model-path ~/.local/share/parakeet-cli/models/parakeet-tdt-0.6b-v3-int8\n"
            );
            eprintln!("Or set the LOCAL_PARAKEET_MODEL_PATH environment variable.");
            eprintln!();
            eprintln!("Tip: Run 'whis setup' for guided setup.");
            std::process::exit(1);
        }
    };

    Ok(TranscriptionConfig {
        provider,
        model_path,
        language,
    })
}

/// Load transcription config using configured language
pub fn load_transcription_config() -> Result<TranscriptionConfig> {
    load_transcription_config_with_language(None)
}

/// Wait for user to stop recording via Enter key.
/// In TTY mode: waits for Enter key press.
/// In non-TTY mode: blocks indefinitely (use --duration for timed recording).
pub fn wait_for_stop() -> Result<()> {
    std::io::stdout().flush()?;

    if std::io::stdin().is_terminal() {
        enable_raw_mode()?;

        loop {
            if event::poll(Duration::from_millis(50))?
                && let Event::Key(key_event) = event::read()?
                && key_event.code == KeyCode::Enter
            {
                break;
            }
        }

        disable_raw_mode()?;
    } else {
        loop {
            thread::sleep(Duration::from_secs(3600));
        }
    }

    Ok(())
}

/// Print text with a typewriter effect
pub fn typewriter(text: &str, delay_ms: u64) {
    if delay_ms == 0 {
        print!("{}", text);
        std::io::stdout().flush().ok();
        return;
    }
    for c in text.chars() {
        print!("{}", c);
        std::io::stdout().flush().ok();
        thread::sleep(Duration::from_millis(delay_ms));
    }
}

/// Print status message
pub fn print_status(message: &str, _provider: Option<&TranscriptionProvider>) {
    if whis_core::verbose::is_verbose() {
        println!("{}", message.trim());
        return;
    }
    typewriter(message, 25);
}
