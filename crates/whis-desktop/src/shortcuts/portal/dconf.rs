//! GNOME dconf Shortcut Reading
//!
//! Provides functionality to read shortcuts from GNOME's dconf database:
//! - **Portal shortcuts**: `/org/gnome/settings-daemon/global-shortcuts/` (XDG Portal)
//! - **Custom shortcuts**: `/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/` (GNOME Settings)

/// Read the actual portal shortcut from dconf (GNOME)
/// Returns the shortcut in format like "Ctrl+Alt+M" if found
#[cfg(target_os = "linux")]
pub fn read_portal_shortcut_from_dconf() -> Option<String> {
    // Run: dconf dump /org/gnome/settings-daemon/global-shortcuts/
    let output = std::process::Command::new("dconf")
        .args(["dump", "/org/gnome/settings-daemon/global-shortcuts/"])
        .output()
        .ok()?;

    let dump = String::from_utf8_lossy(&output.stdout);

    // Look for toggle-recording in any app section
    // Format: shortcuts=[('toggle-recording', {'shortcuts': <['<Control><Alt>m']>, ...})]
    for line in dump.lines() {
        if line.contains("toggle-recording") && line.contains("shortcuts") {
            // Parse the GVariant format: <['<Control><Alt>m']>
            if let Some(start) = line.find("<['")
                && let Some(end) = line[start..].find("']>")
            {
                let raw = &line[start + 3..start + end];
                // Convert <Control><Alt>m to Ctrl+Alt+M
                return Some(convert_gvariant_shortcut(raw));
            }
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
pub fn read_portal_shortcut_from_dconf() -> Option<String> {
    None
}

/// Convert GVariant shortcut format to human-readable format
/// e.g., "<Control><Alt>m" -> "Ctrl+Alt+M"
#[cfg(target_os = "linux")]
fn convert_gvariant_shortcut(raw: &str) -> String {
    let converted = raw
        .replace("<Control>", "Ctrl+")
        .replace("<Alt>", "Alt+")
        .replace("<Shift>", "Shift+")
        .replace("<Super>", "Super+");

    // Uppercase the final key and handle trailing +
    if let Some(last_plus) = converted.rfind('+') {
        let (modifiers, key) = converted.split_at(last_plus + 1);
        format!("{}{}", modifiers, key.to_uppercase())
    } else {
        converted.to_uppercase()
    }
}

/// Read custom shortcut from GNOME Settings that points to whis-desktop --toggle
///
/// Scans `/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/`
/// for any shortcut whose command contains "whis" and "--toggle".
///
/// Returns the shortcut in format like "Ctrl+Alt+F" if found.
#[cfg(target_os = "linux")]
pub fn read_gnome_custom_shortcut() -> Option<String> {
    let output = std::process::Command::new("dconf")
        .args([
            "dump",
            "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/",
        ])
        .output()
        .ok()?;

    let dump = String::from_utf8_lossy(&output.stdout);

    let mut current_binding: Option<String> = None;
    let mut found_whis_command = false;

    for line in dump.lines() {
        // New section starts - check if previous section was a whis command
        if line.starts_with('[') {
            if found_whis_command && let Some(binding) = current_binding.take() {
                return Some(convert_gvariant_shortcut(&binding));
            }
            // Reset for new section
            current_binding = None;
            found_whis_command = false;
            continue;
        }

        // Track binding for current section: binding='<Control><Alt>f'
        if let Some(rest) = line.strip_prefix("binding='") {
            current_binding = rest.strip_suffix('\'').map(|s| s.to_string());
        }

        // Check if command contains whis and --toggle
        if line.starts_with("command=") {
            let cmd = line
                .trim_start_matches("command=")
                .trim_matches(|c| c == '\'' || c == '"');
            if cmd.to_lowercase().contains("whis") && cmd.contains("--toggle") {
                found_whis_command = true;
            }
        }
    }

    // Check final section (no trailing '[' to trigger)
    if found_whis_command && let Some(binding) = current_binding {
        return Some(convert_gvariant_shortcut(&binding));
    }

    None
}

#[cfg(not(target_os = "linux"))]
pub fn read_gnome_custom_shortcut() -> Option<String> {
    None
}

/// Read the command from GNOME custom shortcut that points to whis-desktop --toggle
///
/// Scans `/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/`
/// for any shortcut whose command contains "whis" and "--toggle".
///
/// Returns the full command string like "/path/to/whis-desktop --toggle" if found.
#[cfg(target_os = "linux")]
pub fn read_gnome_custom_shortcut_command() -> Option<String> {
    let output = std::process::Command::new("dconf")
        .args([
            "dump",
            "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/",
        ])
        .output()
        .ok()?;

    let dump = String::from_utf8_lossy(&output.stdout);

    for line in dump.lines() {
        if line.starts_with("command=") {
            let cmd = line
                .trim_start_matches("command=")
                .trim_matches(|c| c == '\'' || c == '"');
            if cmd.to_lowercase().contains("whis") && cmd.contains("--toggle") {
                return Some(cmd.to_string());
            }
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
pub fn read_gnome_custom_shortcut_command() -> Option<String> {
    None
}
