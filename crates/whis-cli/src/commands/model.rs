//! Model listing commands for Parakeet

use anyhow::Result;
use whis_core::model::{ModelType, ParakeetModel};

use crate::args::ModelAction;

/// Run the model command
pub fn run(action: Option<ModelAction>) -> Result<()> {
    match action {
        None | Some(ModelAction::List) => list_parakeet_models(),
    }
}

/// List available Parakeet models with install status
fn list_parakeet_models() -> Result<()> {
    println!("Available Parakeet models:\n");

    let name_width = ParakeetModel
        .models()
        .iter()
        .map(|model| model.name.len())
        .max()
        .unwrap_or(6)
        .max(4);

    println!(
        "{:<name_width$}  STATUS       DESCRIPTION",
        "NAME",
        name_width = name_width
    );
    println!("{}", "-".repeat(60));

    for model in ParakeetModel.models() {
        let path = ParakeetModel.default_path(model.name);
        let status = if ParakeetModel.verify(&path) {
            "[installed]"
        } else {
            ""
        };

        println!(
            "{:<name_width$}  {:<11}  {}",
            model.name,
            status,
            model.description,
            name_width = name_width
        );
    }

    println!();
    println!("Models directory: {}", ParakeetModel.default_dir().display());
    println!();
    println!("To download a model, run: whis setup");

    Ok(())
}
