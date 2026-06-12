use clap::{Args, Parser, Subcommand, ValueHint};
use std::time::Duration;

/// Parse a duration string like "10s", "30s", "1m", "90"
fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("duration cannot be empty".to_string());
    }

    if let Some(num_str) = s.strip_suffix('s') {
        let secs: u64 = num_str
            .parse()
            .map_err(|_| format!("invalid number: {}", num_str))?;
        Ok(Duration::from_secs(secs))
    } else if let Some(num_str) = s.strip_suffix('m') {
        let mins: u64 = num_str
            .parse()
            .map_err(|_| format!("invalid number: {}", num_str))?;
        Ok(Duration::from_secs(mins * 60))
    } else {
        let secs: u64 = s.parse().map_err(|_| format!("invalid duration: {}", s))?;
        Ok(Duration::from_secs(secs))
    }
}

/// Input options for transcription
#[derive(Args)]
pub struct InputOptions {
    /// Transcribe an audio file instead of recording from microphone
    #[arg(short = 'f', long, value_name = "PATH", value_hint = ValueHint::FilePath)]
    pub file: Option<std::path::PathBuf>,
}

/// Processing options for transcription
#[derive(Args)]
pub struct ProcessingOptions {
    /// Record for a fixed duration (e.g., "10s", "30s", "1m")
    #[arg(short = 'd', long, value_parser = parse_duration)]
    pub duration: Option<Duration>,

    /// Disable Voice Activity Detection (records all audio including silence)
    #[arg(long)]
    pub no_vad: bool,

    /// Language code for transcription (e.g., "en", "de", "fr", "auto")
    /// Overrides the configured language for this invocation only
    #[arg(short = 'l', long)]
    pub language: Option<String>,
}

/// Output format for transcription
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Plain text (default)
    #[default]
    Txt,
    /// SubRip subtitle format
    Srt,
    /// WebVTT subtitle format
    Vtt,
}

impl OutputFormat {
    /// Detect format from file extension
    pub fn from_extension(path: &std::path::Path) -> Option<Self> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("srt") => Some(Self::Srt),
            Some("vtt") => Some(Self::Vtt),
            _ => None,
        }
    }
}

/// Output options for transcription results
#[derive(Args)]
pub struct OutputOptions {
    /// Print output to stdout instead of copying to clipboard
    #[arg(long)]
    pub print: bool,

    /// Save output to file instead of copying to clipboard
    #[arg(short = 'o', long, value_name = "PATH", value_hint = ValueHint::FilePath)]
    pub output: Option<std::path::PathBuf>,

    /// Output format (txt, srt, vtt)
    #[arg(long, value_enum, default_value = "txt")]
    pub format: OutputFormat,
}

#[derive(Parser)]
#[command(name = "whis")]
#[command(version)]
#[command(about = "Local Parakeet ASR CLI - transcribe speech from your microphone")]
#[command(after_help = "Run 'whis' without arguments to record once (press Enter to stop).")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output for debugging (audio device, clipboard, etc.)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(flatten)]
    pub input: InputOptions,

    #[command(flatten)]
    pub processing: ProcessingOptions,

    #[command(flatten)]
    pub output: OutputOptions,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// Start the background service (uses shortcut_mode from settings)
    Start {
        /// Override output method to autotype into active window
        #[arg(long)]
        autotype: bool,
    },

    /// Stop the background service
    Stop,

    /// Restart the background service
    Restart {
        /// Override output method to autotype into active window
        #[arg(long)]
        autotype: bool,
    },

    /// Check service status
    Status,

    /// Toggle recording state (for compositor keybindings)
    Toggle,

    /// Interactive setup wizard (downloads Parakeet model if needed)
    Setup,

    /// Configure settings (git-style interface)
    Config {
        key: Option<String>,
        value: Option<String>,

        /// List all configuration settings
        #[arg(long, conflicts_with_all = ["key", "value"])]
        list: bool,

        /// Show configuration file path
        #[arg(long, conflicts_with_all = ["key", "value", "list"])]
        path: bool,
    },

    /// List available Parakeet models with install status
    Model {
        #[command(subcommand)]
        action: Option<ModelAction>,
    },
}

#[derive(Subcommand)]
pub enum ModelAction {
    /// List available Parakeet models with install status
    List,
}
