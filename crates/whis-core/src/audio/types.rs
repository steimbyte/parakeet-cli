//! Core audio types used throughout the audio module.

use serde::{Deserialize, Serialize};

/// Information about an available audio input device.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    /// Device name as reported by the system (used for selection)
    pub name: String,
    /// Cleaned up display name for UI (None = use name)
    #[serde(default)]
    pub display_name: Option<String>,
    /// Whether this is the default input device
    pub is_default: bool,
    /// Device form factor from PulseAudio (e.g., "microphone", "headset", "webcam")
    #[serde(default)]
    pub form_factor: Option<String>,
    /// Device bus type from PulseAudio (e.g., "usb", "pci", "bluetooth")
    #[serde(default)]
    pub bus: Option<String>,
    /// True if this is a monitor source (loopback from output, not a real mic)
    #[serde(default)]
    pub is_monitor: bool,
}
