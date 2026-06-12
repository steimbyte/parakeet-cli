//! Record Command Types

use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;

use crate::args::{InputOptions, OutputFormat, OutputOptions, ProcessingOptions};

/// Configuration for the record command
#[derive(Debug, Clone)]
pub struct RecordConfig {
    pub input_file: Option<PathBuf>,
    pub print: bool,
    pub output_path: Option<PathBuf>,
    pub format: OutputFormat,
    pub duration: Option<Duration>,
    pub no_vad: bool,
    pub language: Option<String>,
}

impl RecordConfig {
    pub fn from_cli(
        input: &InputOptions,
        processing: &ProcessingOptions,
        output: &OutputOptions,
    ) -> Result<Self> {
        let format = if output.format == OutputFormat::Txt {
            output
                .output
                .as_ref()
                .and_then(|p| OutputFormat::from_extension(p))
                .unwrap_or(output.format)
        } else {
            output.format
        };

        Ok(Self {
            input_file: input.file.clone(),
            print: output.print,
            output_path: output.output.clone(),
            format,
            duration: processing.duration,
            no_vad: processing.no_vad,
            language: processing.language.clone(),
        })
    }

    pub fn is_quiet(&self) -> bool {
        self.print
    }
}

/// Result of transcription
#[derive(Debug)]
pub struct TranscriptionResult {
    pub text: String,
}
