mod cli;
mod commands;
mod config;
mod emotes;
mod git;
mod llm;
mod ui;

use color_eyre::eyre::Result;

use cli::parse_args;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // Load configuration
    let config = Config::load()?;

    // Parse CLI arguments
    let cli = parse_args();

    // Process commands or default behavior
    match &cli.command {
        Some(command) => {
            commands::handle_commands(command, &config).await?;
        }
        None => {
            // No subcommand provided, default to generate behavior using cli directly
            commands::generate_commit(&config, cli.auto_add, cli.auto_commit, cli.auto_push)
                .await?;
        }
    }

    Ok(())
}
