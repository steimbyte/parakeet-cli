//! Setup wizard for parakeet-cli.
//!
//! Single flow: pick a Parakeet model, download if needed, save path.

mod interactive;
mod local;

use anyhow::Result;

use whis_core::Settings;

pub fn run() -> Result<()> {
    setup_wizard()
}

fn setup_wizard() -> Result<()> {
    local::setup_transcription_local()?;

    // Audio device selection step
    setup_audio_device_step()?;

    // Shortcut setup step
    setup_shortcut_step()?;

    interactive::info("Configuration saved! Run 'whis' to record and transcribe.");
    Ok(())
}

fn setup_shortcut_step() -> Result<()> {
    use whis_core::settings::CliShortcutMode;

    let mut settings = Settings::load_cli();

    let items = vec!["System shortcut", "Direct capture"];
    let default = if settings.shortcuts.cli_mode == CliShortcutMode::Direct {
        1
    } else {
        0
    };
    let choice = interactive::select("Recording trigger?", &items, Some(default))?;

    match choice {
        0 => {
            settings.shortcuts.cli_mode = CliShortcutMode::System;
            settings.save_cli()?;
            interactive::info("Add a shortcut in your desktop environment that runs: whis toggle");
        }
        1 => {
            settings.shortcuts.cli_mode = CliShortcutMode::Direct;
            let default_shortcut = &settings.shortcuts.cli_key;
            let normalized = loop {
                let input = interactive::input("Hotkey?", Some(default_shortcut))?;
                match crate::hotkey::validate(&input) {
                    Ok(n) => {
                        settings.shortcuts.cli_key = input;
                        settings.save_cli()?;
                        break n;
                    }
                    Err(_) => {
                        interactive::error("Invalid hotkey. Examples: ctrl+alt+p, super+shift+r");
                        continue;
                    }
                }
            };
            #[cfg(target_os = "linux")]
            {
                if is_in_input_group() {
                    interactive::info(&format!("Hotkey {} ready", normalized));
                } else {
                    interactive::info(&format!("Valid: {}", normalized));
                    interactive::error("You need permission first:");
                    interactive::info("  sudo usermod -aG input $USER");
                    interactive::info("Then logout and login again.");
                }
            }
            #[cfg(not(target_os = "linux"))]
            interactive::info(&format!("Hotkey {} ready", normalized));
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn setup_audio_device_step() -> Result<()> {
    use whis_core::list_audio_devices;

    let devices = match list_audio_devices() {
        Ok(d) if !d.is_empty() => d,
        _ => return Ok(()),
    };

    let mut items: Vec<String> = vec!["System Default".to_string()];
    for device in &devices {
        let name = device.display_name.as_ref().unwrap_or(&device.name);
        items.push(name.to_string());
    }

    let mut settings = Settings::load_cli();
    let default_idx = settings
        .ui
        .microphone_device
        .as_ref()
        .and_then(|current| devices.iter().position(|d| &d.name == current))
        .map(|i| i + 1)
        .unwrap_or(0);

    let choice = interactive::select("Microphone?", &items, Some(default_idx))?;

    settings.ui.microphone_device = if choice == 0 {
        None
    } else {
        Some(devices[choice - 1].name.clone())
    };

    settings.save_cli()?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn is_in_input_group() -> bool {
    use std::process::Command;
    Command::new("groups")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("input"))
        .unwrap_or(false)
}
