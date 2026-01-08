//! Server command handler

use crate::ServerCommands;
use pctrl_core::{Server, ServerType};
use pctrl_database::Database;

pub async fn handle(command: ServerCommands, db: &Database) -> anyhow::Result<()> {
    match command {
        ServerCommands::List => {
            let servers = db.list_servers().await?;
            if servers.is_empty() {
                println!("No servers configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl server add <name> <host> [-t type] [-p provider]");
            } else {
                println!("Servers ({}):", servers.len());
                println!();
                for server in servers {
                    let provider_str = server
                        .provider
                        .map(|p| format!(" ({})", p))
                        .unwrap_or_default();
                    println!(
                        "  🖥️  {} - {} [{}]{}",
                        server.name, server.host, server.server_type, provider_str
                    );
                }
            }
        }

        ServerCommands::Add {
            name,
            host,
            server_type,
            provider,
            ssh,
            location,
        } => {
            let id = name.to_lowercase().replace(' ', "-");

            if db.get_server_by_name(&name).await?.is_some() {
                anyhow::bail!("Server '{}' already exists.", name);
            }

            let server_type: ServerType = server_type.parse().unwrap_or_default();

            let server = Server {
                id: id.clone(),
                name: name.clone(),
                host: host.clone(),
                server_type: server_type.clone(),
                provider: provider.clone(),
                ssh_connection_id: ssh,
                location,
                specs: None,
                notes: None,
            };

            db.save_server(&server).await?;

            println!("✓ Server added:");
            println!();
            println!("  Name:     {}", name);
            println!("  ID:       {}", id);
            println!("  Host:     {}", host);
            println!("  Type:     {}", server_type);
            if let Some(p) = provider {
                println!("  Provider: {}", p);
            }
        }

        ServerCommands::Show { name } => {
            let server = db
                .get_server_by_name(&name)
                .await?
                .or(db.get_server(&name).await?)
                .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", name))?;

            println!();
            println!("  🖥️  {}", server.name);
            println!("  ─────────────────────────────");
            println!("  ID:       {}", server.id);
            println!("  Host:     {}", server.host);
            println!("  Type:     {}", server.server_type);
            if let Some(p) = &server.provider {
                println!("  Provider: {}", p);
            }
            if let Some(l) = &server.location {
                println!("  Location: {}", l);
            }
            if let Some(ssh) = &server.ssh_connection_id {
                println!("  SSH:      {}", ssh);
            }
            println!();
        }

        ServerCommands::Remove { name } => {
            let server = db
                .get_server_by_name(&name)
                .await?
                .or(db.get_server(&name).await?)
                .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", name))?;

            if db.remove_server(&server.id).await? {
                println!("✓ Server '{}' removed", server.name);
            }
        }
    }

    Ok(())
}
