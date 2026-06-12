mod app;
mod args;
mod commands;
mod error;
mod hotkey;
mod ipc;
mod service;
mod ui;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    if let Err(err) = run() {
        error::display_anyhow_error(err);
        std::process::exit(1);
    }
    Ok(())
}

fn run() -> Result<()> {
    let cli = args::Cli::parse();
    whis_core::set_verbose(cli.verbose);

    match cli.command {
        Some(args::Commands::Start { autotype }) => commands::start::run(autotype),
        Some(args::Commands::Stop) => commands::stop::run(),
        Some(args::Commands::Restart { autotype }) => commands::restart::run(autotype),
        Some(args::Commands::Status) => commands::status::run(),
        Some(args::Commands::Toggle) => commands::toggle::run(),
        Some(args::Commands::Config {
            key,
            value,
            list,
            path,
        }) => commands::config::run(key, value, list, path),
        Some(args::Commands::Setup) => commands::setup::run(),
        Some(args::Commands::Model { action }) => commands::model::run(action),
        None => {
            let config =
                commands::record::RecordConfig::from_cli(&cli.input, &cli.processing, &cli.output)?;
            commands::record::run(config)
        }
    }
}
