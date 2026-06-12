//! Shortcut Backend Detection
//!
//! Detects the appropriate global shortcut backend for the current platform:
//! - TauriPlugin: X11, macOS, Windows
//! - RdevGrab: Linux via evdev (requires input group permissions)
//! - PortalGlobalShortcuts: Wayland with portal support (GNOME 48+, KDE, Hyprland)
//! - ManualSetup: Fallback to IPC (`whis-desktop --toggle`)

use serde::Serialize;
use whis_core::platform::{Platform, PlatformInfo, detect_platform};

/// Backend for global keyboard shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum ShortcutBackend {
    /// Tauri plugin - works on X11, macOS, Windows
    TauriPlugin,
    /// rdev::grab() - works on Linux (X11 and Wayland) via evdev
    RdevGrab,
    /// XDG Portal GlobalShortcuts - works on Wayland with GNOME 48+, KDE, Hyprland
    PortalGlobalShortcuts,
    /// Manual setup - user configures compositor to run `whis-desktop --toggle`
    ManualSetup,
}

/// Information about shortcut capability on current system
pub struct ShortcutCapability {
    pub backend: ShortcutBackend,
    pub platform_info: PlatformInfo,
}

/// Backend info for frontend consumption
#[derive(Debug, Clone, Serialize)]
pub struct ShortcutBackendInfo {
    pub backend: String,
    pub requires_restart: bool,
    pub compositor: String,
    pub portal_version: u32,
    pub is_flatpak: bool,
}

/// Get the GlobalShortcuts portal version (0 if unavailable)
pub fn portal_version() -> u32 {
    detect_platform().portal_version
}

/// Get backend info for the frontend
pub fn backend_info() -> ShortcutBackendInfo {
    let capability = detect_backend();

    // RdevGrab and Portal require restart to update shortcuts
    let requires_restart = !matches!(capability.backend, ShortcutBackend::TauriPlugin);

    ShortcutBackendInfo {
        backend: format!("{:?}", capability.backend),
        requires_restart,
        compositor: capability
            .platform_info
            .compositor
            .display_name()
            .to_string(),
        portal_version: capability.platform_info.portal_version,
        is_flatpak: capability.platform_info.is_flatpak,
    }
}

/// Detect the best shortcut backend for the current environment
pub fn detect_backend() -> ShortcutCapability {
    let platform_info = detect_platform();

    let backend = match platform_info.platform {
        Platform::MacOS | Platform::Windows => ShortcutBackend::TauriPlugin,
        Platform::LinuxX11 => ShortcutBackend::TauriPlugin,
        Platform::LinuxWayland => {
            if platform_info.is_flatpak {
                // Flatpak sandbox blocks /dev/input/* access
                // Must use Portal or manual compositor config
                if platform_info.portal_version >= 1 {
                    ShortcutBackend::PortalGlobalShortcuts
                } else {
                    ShortcutBackend::ManualSetup
                }
            } else {
                // Non-sandboxed: use rdev::grab() via evdev
                ShortcutBackend::RdevGrab
            }
        }
    };

    ShortcutCapability {
        backend,
        platform_info,
    }
}
