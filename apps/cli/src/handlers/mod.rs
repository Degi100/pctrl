//! Command handlers for the CLI
//!
//! Each module handles a specific command group.

mod database;
mod domain;
mod project;
mod script;
mod server;

use crate::Commands;
use pctrl_database::Database;
use std::sync::Arc;

/// Main command dispatcher
pub async fn handle_command(command: Commands, db: Arc<Database>) -> anyhow::Result<()> {
    match command {
        Commands::Project { command } => project::handle(command, &db).await,
        Commands::Server { command } => server::handle(command, &db).await,
        Commands::Domain { command } => domain::handle(command, &db).await,
        Commands::Database { command } => database::handle(command, &db).await,
        Commands::Script { command } => script::handle(command, &db).await,
    }
}
