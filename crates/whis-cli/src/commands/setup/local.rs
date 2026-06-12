//! Parakeet (on-device) transcription setup

use anyhow::Result;
use whis_core::Settings;
use whis_core::model::{self, ModelType, ParakeetModel};

use super::interactive;

pub fn setup_transcription_local() -> Result<()> {
    let mut settings = Settings::load_cli();

    // Build selection items with markers
    let current_model = settings
        .transcription
        .local_models
        .parakeet_path
        .as_ref()
        .and_then(|p| {
            ParakeetModel
                .models()
                .iter()
                .find(|m| ParakeetModel.default_path(m.name) == *p)
                .map(|m| m.name.to_string())
        });

    let (items, clean_items): (Vec<String>, Vec<String>) = ParakeetModel
        .models()
        .iter()
        .map(|model| {
            let path = ParakeetModel.default_path(model.name);
            let installed = if ParakeetModel.verify(&path) {
                " [installed]"
            } else {
                ""
            };
            let current = if current_model.as_deref() == Some(model.name) {
                " [current]"
            } else {
                ""
            };
            (
                format!("{}{}{}", model.name, installed, current),
                model.name.to_string(),
            )
        })
        .unzip();

    let default_idx = current_model
        .as_deref()
        .and_then(|name| ParakeetModel.models().iter().position(|m| m.name == name))
        .unwrap_or(0);

    let model_choice = interactive::select_clean(
        "Which Parakeet model?",
        &items,
        &clean_items,
        Some(default_idx),
    )?;
    let model = &ParakeetModel.models()[model_choice];

    let path = ParakeetModel.default_path(model.name);
    if !ParakeetModel.verify(&path) {
        interactive::info(&format!("Downloading {}...", model.name));
        model::download::download(&ParakeetModel, model.name, &path)?;
    }

    settings.transcription.provider = whis_core::TranscriptionProvider::LocalParakeet;
    settings.transcription.local_models.parakeet_path =
        Some(path.to_string_lossy().to_string());
    settings.save_cli()?;

    Ok(())
}
