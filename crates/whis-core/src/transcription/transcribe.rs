//! Progressive audio transcription using parakeet (local).
//!
//! All audio inputs (microphone, file, stdin) use progressive transcription
//! via the local parakeet model. Supports overlap merging for seamless chunk boundaries.

use anyhow::{Context, Result};

use crate::audio::chunker::AudioChunk as ProgressiveChunk;

/// Maximum words to search for overlap between chunks
const MAX_OVERLAP_WORDS: usize = 15;

/// Result of transcribing a single chunk
struct ChunkTranscription {
    index: usize,
    text: String,
    has_leading_overlap: bool,
}

/// Merge transcription results, handling overlaps
fn merge_transcriptions(transcriptions: Vec<ChunkTranscription>) -> String {
    if transcriptions.is_empty() {
        return String::new();
    }

    if transcriptions.len() == 1 {
        return transcriptions.into_iter().next().unwrap().text;
    }

    let mut merged = String::new();

    for (i, transcription) in transcriptions.into_iter().enumerate() {
        let text = transcription.text.trim();

        if i == 0 {
            merged.push_str(text);
        } else if transcription.has_leading_overlap {
            let cleaned_text = remove_overlap(&merged, text);

            if cleaned_text.trim().is_empty() {
                crate::verbose!(
                    "Chunk {} completely deduplicated after overlap removal",
                    transcription.index
                );
                continue;
            }

            if !merged.ends_with(' ') && !cleaned_text.is_empty() && !cleaned_text.starts_with(' ')
            {
                merged.push(' ');
            }
            merged.push_str(&cleaned_text);
        } else {
            if !merged.ends_with(' ') && !text.is_empty() && !text.starts_with(' ') {
                merged.push(' ');
            }
            merged.push_str(text);
        }
    }

    merged
}

/// Remove overlapping text from the beginning of new_text that matches end of existing_text
fn remove_overlap(existing: &str, new_text: &str) -> String {
    let existing_words: Vec<&str> = existing.split_whitespace().collect();
    let new_words: Vec<&str> = new_text.split_whitespace().collect();

    if existing_words.is_empty() || new_words.is_empty() {
        return new_text.to_string();
    }

    let search_end = existing_words.len().min(MAX_OVERLAP_WORDS);
    let search_new = new_words.len().min(MAX_OVERLAP_WORDS);

    let mut best_overlap = 0;

    for overlap_len in 1..=search_end.min(search_new) {
        let end_slice = &existing_words[existing_words.len() - overlap_len..];
        let start_slice = &new_words[..overlap_len];

        let matches = end_slice
            .iter()
            .zip(start_slice.iter())
            .all(|(a, b)| a.eq_ignore_ascii_case(b));

        if matches {
            best_overlap = overlap_len;
        }
    }

    if best_overlap > 0 {
        new_words[best_overlap..].join(" ")
    } else {
        new_text.to_string()
    }
}

/// Progressive transcription using the local parakeet model.
///
/// Transcribes audio chunks DURING recording. As each chunk is produced,
/// it's immediately transcribed using the shared cached model (sequential
/// processing). The model is loaded once and reused to minimize memory usage.
#[cfg(feature = "local-transcription")]
pub async fn progressive_transcribe_local(
    model_path: &str,
    mut chunk_rx: tokio::sync::mpsc::UnboundedReceiver<ProgressiveChunk>,
    progress_callback: Option<Box<dyn Fn(usize, usize) + Send + Sync>>,
) -> Result<String> {
    let mut transcriptions = Vec::new();
    let mut chunk_count = 0;

    while let Some(chunk) = chunk_rx.recv().await {
        chunk_count += 1;
        let chunk_index = chunk.index;
        let has_leading_overlap = chunk.has_leading_overlap;
        let samples = chunk.samples;
        let model_path_owned = model_path.to_string();

        let result = tokio::task::spawn_blocking(move || {
            crate::provider::transcribe_raw_parakeet(&model_path_owned, samples)
        })
        .await
        .context("Transcription task panicked")?
        .context("Transcription failed")?;

        transcriptions.push(ChunkTranscription {
            index: chunk_index,
            text: result.text,
            has_leading_overlap,
        });

        if let Some(ref callback) = progress_callback {
            callback(chunk_count, 0);
        }
    }

    Ok(merge_transcriptions(transcriptions))
}
