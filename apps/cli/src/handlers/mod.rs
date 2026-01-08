//! Command handlers for the CLI
//!
//! Each module handles a specific command group.

mod database;
mod domain;
mod project;
mod script;
mod server;

// Legacy command handlers
pub mod legacy;

use crate::Commands;
use pctrl_core::Config;
use pctrl_database::Database;
use std::sync::Arc;

/// Main command dispatcher
pub async fn handle_command(
    command: Commands,
    config: Arc<Config>,
    db: Arc<Database>,
) -> anyhow::Result<()> {
    match command {
        // v6: Project-centric commands
        Commands::Project { command } => project::handle(command, &db).await,
        Commands::Server { command } => server::handle(command, &db).await,
        Commands::Domain { command } => domain::handle(command, &db).await,
        Commands::Database { command } => database::handle(command, &db).await,
        Commands::Script { command } => script::handle(command, &config, &db).await,
        // Legacy commands
        Commands::Ssh { command } => legacy::ssh::handle(command, &config, &db).await,
        Commands::Docker { command } => legacy::docker::handle(command, &config, &db).await,
        Commands::Coolify { command } => legacy::coolify::handle(command, &config, &db).await,
        Commands::Git { command } => legacy::git::handle(command, &config, &db).await,
    }
}
