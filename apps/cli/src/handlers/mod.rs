//! Command handlers for the CLI
//!
//! Each module handles a specific command group.

mod credential;
mod database;
mod domain;
mod project;
mod script;
mod server;

use crate::{Commands, CredentialCommands};
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
        Commands::Credential { command } => handle_credential(command, &db).await,
    }
}

/// Handle credential commands
async fn handle_credential(command: CredentialCommands, db: &Database) -> anyhow::Result<()> {
    match command {
        CredentialCommands::List => credential::handle_list(db).await,
        CredentialCommands::Add {
            name,
            cred_type,
            user,
            port,
            key,
            token,
            password,
            url,
        } => {
            credential::handle_add(db, name, cred_type, user, port, key, token, password, url).await
        }
        CredentialCommands::Show { name } => credential::handle_show(db, name).await,
        CredentialCommands::Remove { name } => credential::handle_remove(db, name).await,
    }
}
