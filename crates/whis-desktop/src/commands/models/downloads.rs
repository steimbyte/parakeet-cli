//! Model Download Synchronization + shared DownloadProgress type

use std::sync::{Mutex, OnceLock};

static WHISPER_DOWNLOAD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static PARAKEET_DOWNLOAD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn get_whisper_lock() -> &'static Mutex<()> {
    WHISPER_DOWNLOAD_LOCK.get_or_init(|| Mutex::new(()))
}

pub fn get_parakeet_lock() -> &'static Mutex<()> {
    PARAKEET_DOWNLOAD_LOCK.get_or_init(|| Mutex::new(()))
}

/// Progress event payload emitted to the frontend during model downloads
#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
}
