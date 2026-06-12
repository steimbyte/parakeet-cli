//! Logging macros for consistent output across whis crates.
//!
//! # Macros
//!
//! - `verbose!()` - Debug info, only shown when verbose mode enabled
//! - `info!()` - General information messages
//! - `warn!()` - Warning messages
//! - `error!()` - Error messages
//!
//! # Usage
//!
//! ```ignore
//! use whis_core::{verbose, info, warn, error};
//!
//! verbose!("Debug details: {}", value);  // Only if set_verbose(true)
//! info!("Processing file: {}", path);
//! warn!("Deprecated option used");
//! error!("Failed to connect: {}", err);
//! ```

use std::io::{self, IsTerminal, Write};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

static VERBOSE: AtomicBool = AtomicBool::new(false);
static STDOUT_LOCK: Mutex<()> = Mutex::new(());

/// Enable or disable verbose logging
pub fn set_verbose(enabled: bool) {
    VERBOSE.store(enabled, Ordering::SeqCst);
}

/// Check if verbose logging is enabled
pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::SeqCst)
}

/// Write a verbose message atomically to stdout.
/// Uses a mutex to prevent interleaving from concurrent threads.
///
/// Note: When the CLI enters raw mode (via crossterm), some terminals stop
/// translating `\n` into CRLF. If we only print `\n`, subsequent lines can start
/// at the previous cursor column, causing "stair-step" indentation. To keep
/// output aligned, we emit `\r\n` when stdout is a TTY.
pub fn write_verbose(args: std::fmt::Arguments) {
    let _guard = STDOUT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let is_tty = io::stdout().is_terminal();

    let mut stdout = io::stdout().lock();
    if is_tty {
        let _ = write!(stdout, "[verbose] {}\r\n", args);
    } else {
        let _ = writeln!(stdout, "[verbose] {}", args);
    }
    let _ = stdout.flush();
}

/// Log a formatted message if verbose mode is enabled.
/// Outputs to stdout atomically to prevent interleaving from concurrent threads.
#[macro_export]
macro_rules! verbose {
    ($($arg:tt)*) => {
        if $crate::verbose::is_verbose() {
            $crate::verbose::write_verbose(format_args!($($arg)*));
        }
    };
}

/// Log an info message (always printed)
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        eprintln!("[info] {}", format!($($arg)*));
    };
}

/// Log a warning message (always printed)
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!("[warn] {}", format!($($arg)*));
    };
}

/// Log an error message (always printed)
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("[error] {}", format!($($arg)*));
    };
}
