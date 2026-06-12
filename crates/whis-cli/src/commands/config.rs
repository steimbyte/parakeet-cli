use anyhow::{Context, Result, anyhow};
use whis_core::settings::CliShortcutMode;
use whis_core::{Settings, TranscriptionProvider};

use crate::ui::mask_key;

/// Supported configuration keys (parakeet-cli: parakeet + VAD + UI + shortcuts only)
const VALID_KEYS: &[&str] = &[
    "provider",
    "language",
    "parakeet-model-path",
    "microphone-device",
    "cli-mode",
    "cli-key",
    "cli-push-to-talk",
    "desktop-key",
    "vad",
    "vad-threshold",
    "chunk-size",
    "keep-model-loaded",
    "model-unload-minutes",
];

pub fn run(key: Option<String>, value: Option<String>, list: bool, path: bool) -> Result<()> {
    if path {
        println!("{}", Settings::cli_path().display());
        return Ok(());
    }
    if list {
        return show_all_settings();
    }

    if let Some(key_str) = key {
        let key_normalized = key_str.to_lowercase();
        if !VALID_KEYS.contains(&key_normalized.as_str()) {
            eprintln!("Error: Unknown configuration key '{}'", key_str);
            eprintln!();
            eprintln!("Valid keys:");
            for k in VALID_KEYS {
                eprintln!("  {}", k);
            }
            eprintln!();
            eprintln!("Run 'whis config --list' to see current values");
            std::process::exit(1);
        }

        if let Some(val) = value {
            set_config(&key_normalized, &val)
        } else {
            get_config(&key_normalized)
        }
    } else {
        show_usage();
        std::process::exit(1);
    }
}

fn set_config(key: &str, value: &str) -> Result<()> {
    let mut settings = Settings::load_cli();
    let value_trimmed = value.trim();

    match key {
        "provider" => {
            let provider = value_trimmed
                .parse::<TranscriptionProvider>()
                .map_err(|e| anyhow!("{}", e))?;
            settings.transcription.provider = provider;
            println!("provider = {}", value_trimmed);
        }
        "language" => {
            if value_trimmed.to_lowercase() == "auto" {
                settings.transcription.language = None;
                println!("language = auto-detect");
            } else {
                let lang_lower = value_trimmed.to_lowercase();
                if lang_lower.len() != 2 || !lang_lower.chars().all(|c| c.is_ascii_lowercase()) {
                    anyhow::bail!(
                        "Invalid language code. Use ISO-639-1 format (e.g., 'en', 'de', 'fr') or 'auto'"
                    );
                }
                settings.transcription.language = Some(lang_lower.clone());
                println!("language = {}", lang_lower);
            }
        }
        "parakeet-model-path" => {
            if value_trimmed.is_empty() {
                anyhow::bail!("Invalid parakeet model path: cannot be empty");
            }
            let expanded_path = expand_home_dir(value_trimmed);
            settings.transcription.local_models.parakeet_path = Some(expanded_path.clone());
            println!("parakeet-model-path = {}", expanded_path);
        }
        "microphone-device" => {
            if value_trimmed.to_lowercase() == "default" || value_trimmed.is_empty() {
                settings.ui.microphone_device = None;
                println!("microphone-device = System Default");
            } else {
                settings.ui.microphone_device = Some(value_trimmed.to_string());
                println!("microphone-device = {}", value_trimmed);
            }
        }
        "vad" => {
            let enabled = value_trimmed
                .parse::<bool>()
                .context("Invalid value. Use 'true' or 'false'")?;
            settings.ui.vad.enabled = enabled;
            println!("vad = {}", enabled);
        }
        "vad-threshold" => {
            let threshold = value_trimmed
                .parse::<f32>()
                .context("Invalid threshold. Use a number between 0.0 and 1.0")?;
            if !(0.0..=1.0).contains(&threshold) {
                anyhow::bail!("Invalid VAD threshold: must be between 0.0 and 1.0");
            }
            settings.ui.vad.threshold = threshold;
            println!("vad-threshold = {:.2}", threshold);
        }
        "chunk-size" => {
            let size = value_trimmed
                .parse::<u64>()
                .context("Invalid chunk size. Use a number of seconds (e.g., 30, 60, 90)")?;
            if !(10..=300).contains(&size) {
                anyhow::bail!("Invalid chunk size: must be between 10 and 300 seconds");
            }
            settings.ui.chunk_duration_secs = size;
            println!("chunk-size = {}s", size);
        }
        "cli-mode" => {
            let mode: CliShortcutMode = value_trimmed
                .parse()
                .map_err(|e: String| anyhow!("{}", e))?;
            settings.shortcuts.cli_mode = mode;
            settings.shortcuts.validate()?;
            println!("cli-mode = {}", mode);
        }
        "cli-key" => {
            if value_trimmed.is_empty() {
                anyhow::bail!("Invalid CLI shortcut key: cannot be empty");
            }
            settings.shortcuts.cli_key = value_trimmed.to_string();
            settings.shortcuts.validate()?;
            println!("cli-key = {}", value_trimmed);
        }
        "desktop-key" => {
            if value_trimmed.is_empty() {
                anyhow::bail!("Invalid Desktop shortcut key: cannot be empty");
            }
            settings.shortcuts.desktop_key = value_trimmed.to_string();
            settings.shortcuts.validate()?;
            println!("desktop-key = {}", value_trimmed);
        }
        "cli-push-to-talk" => {
            let enabled = value_trimmed
                .parse::<bool>()
                .context("Invalid value. Use 'true' or 'false'")?;
            settings.shortcuts.cli_push_to_talk = enabled;
            println!("cli-push-to-talk = {}", enabled);
        }
        "keep-model-loaded" => {
            let enabled = value_trimmed
                .parse::<bool>()
                .context("Invalid value. Use 'true' or 'false'")?;
            settings.ui.model_memory.keep_model_loaded = enabled;
            println!("keep-model-loaded = {}", enabled);
        }
        "model-unload-minutes" => {
            let mins = value_trimmed
                .parse::<u32>()
                .context("Invalid value. Use a non-negative integer")?;
            settings.ui.model_memory.unload_after_minutes = mins;
            println!("model-unload-minutes = {}", mins);
        }
        _ => unreachable!("Key validation should prevent this"),
    }

    settings.save_cli()?;
    Ok(())
}

fn get_config(key: &str) -> Result<()> {
    let settings = Settings::load_cli();
    match key {
        "provider" => println!("{}", settings.transcription.provider),
        "language" => println!(
            "{}",
            settings.transcription.language.as_deref().unwrap_or("auto")
        ),
        "parakeet-model-path" => {
            if let Some(path) = &settings.transcription.local_models.parakeet_path {
                println!("{}", path);
            } else {
                println!("(not set, using $LOCAL_PARAKEET_MODEL_PATH)");
            }
        }
        "microphone-device" => {
            if let Some(device) = &settings.ui.microphone_device {
                println!("{}", device);
            } else {
                println!("System Default");
            }
        }
        "vad" => println!("{}", settings.ui.vad.enabled),
        "vad-threshold" => println!("{:.2}", settings.ui.vad.threshold),
        "chunk-size" => println!("{}s", settings.ui.chunk_duration_secs),
        "cli-mode" => println!("{}", settings.shortcuts.cli_mode),
        "cli-key" => println!("{}", settings.shortcuts.cli_key),
        "cli-push-to-talk" => println!("{}", settings.shortcuts.cli_push_to_talk),
        "desktop-key" => println!("{}", settings.shortcuts.desktop_key),
        "keep-model-loaded" => println!("{}", settings.ui.model_memory.keep_model_loaded),
        "model-unload-minutes" => println!("{}", settings.ui.model_memory.unload_after_minutes),
        _ => unreachable!("Key validation should prevent this"),
    }
    Ok(())
}

fn show_all_settings() -> Result<()> {
    let settings = Settings::load_cli();

    println!("Configuration file: {}", Settings::cli_path().display());
    println!();
    println!("[Transcription]");
    println!("provider = {}", settings.transcription.provider);
    println!(
        "language = {}",
        settings.transcription.language.as_deref().unwrap_or("auto")
    );

    println!();
    println!("[Local Models]");
    if let Some(path) = &settings.transcription.local_models.parakeet_path {
        println!("parakeet-model-path = {}", path);
    } else {
        println!("parakeet-model-path = (not set, using $LOCAL_PARAKEET_MODEL_PATH)");
    }

    println!();
    println!("[Audio]");
    if let Some(device) = &settings.ui.microphone_device {
        println!("microphone-device = {}", device);
    } else {
        println!("microphone-device = System Default");
    }

    println!();
    println!("[Voice Activity Detection]");
    println!("vad = {}", settings.ui.vad.enabled);
    println!("vad-threshold = {:.2}", settings.ui.vad.threshold);

    println!();
    println!("[Audio Chunking]");
    println!("chunk-size = {}s", settings.ui.chunk_duration_secs);

    println!();
    println!("[Model Memory]");
    println!("keep-model-loaded = {}", settings.ui.model_memory.keep_model_loaded);
    println!(
        "model-unload-minutes = {}",
        settings.ui.model_memory.unload_after_minutes
    );

    println!();
    println!("[Shortcuts]");
    println!("cli-mode = {}", settings.shortcuts.cli_mode);
    println!("cli-key = {}", settings.shortcuts.cli_key);
    println!("cli-push-to-talk = {}", settings.shortcuts.cli_push_to_talk);
    println!("desktop-key = {}", settings.shortcuts.desktop_key);

    Ok(())
}

fn show_usage() {
    eprintln!("Usage:");
    eprintln!("  whis config <key> <value>    Set a configuration value");
    eprintln!("  whis config <key>            Get a configuration value");
    eprintln!("  whis config --list           List all configuration");
    eprintln!("  whis config --path           Show configuration file path");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  whis config provider local-parakeet");
    eprintln!("  whis config parakeet-model-path ~/.local/share/parakeet-cli/models/parakeet-tdt-0.6b-v3-int8");
    eprintln!("  whis config language en");
    eprintln!("  whis config vad true");
    eprintln!("  whis config chunk-size 30");
    eprintln!();
    eprintln!("Run 'whis config --list' to see all available keys and current values");
}

fn expand_home_dir(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest).to_string_lossy().to_string();
    }
    path.to_string()
}

#[allow(dead_code)]
fn placeholder_for_future() {
    let _ = mask_key; // keep import in case future keys are added
}
