//! Server command handler

use crate::ServerCommands;
use pctrl_core::{Server, ServerSpecs, ServerType};
use pctrl_database::Database;
use pctrl_ssh::SshManager;

pub async fn handle(command: ServerCommands, db: &Database) -> anyhow::Result<()> {
    match command {
        ServerCommands::List => {
            let servers = db.list_servers().await?;
            if servers.is_empty() {
                println!("No servers configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl server add <name> <host> [-t type] [-p provider] [--ssh <id>]");
            } else {
                println!("Servers ({}):", servers.len());
                println!();
                for server in servers {
                    let provider_str = server
                        .provider
                        .map(|p| format!(" ({})", p))
                        .unwrap_or_default();
                    let specs_str = server
                        .specs
                        .map(|s| {
                            format!(
                                " [{} CPU, {} GB RAM, {} GB]",
                                s.cpu_cores.map(|c| c.to_string()).unwrap_or("?".into()),
                                s.ram_gb.map(|r| r.to_string()).unwrap_or("?".into()),
                                s.disk_gb.map(|d| d.to_string()).unwrap_or("?".into())
                            )
                        })
                        .unwrap_or_default();
                    println!(
                        "  🖥️  {} - {} [{}]{}{}",
                        server.name, server.host, server.server_type, provider_str, specs_str
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

            // Auto-detect specs via SSH if connection provided
            let specs: Option<ServerSpecs> = if let Some(ref ssh_id) = ssh {
                println!("🔍 Detecting server specs via SSH...");
                match detect_specs_via_ssh(db, ssh_id).await {
                    Ok(specs) => {
                        println!(
                            "  ✓ Detected: {} CPU cores, {} GB RAM, {} GB disk",
                            specs.cpu_cores.map(|c| c.to_string()).unwrap_or("?".into()),
                            specs.ram_gb.map(|r| r.to_string()).unwrap_or("?".into()),
                            specs.disk_gb.map(|d| d.to_string()).unwrap_or("?".into())
                        );
                        Some(specs)
                    }
                    Err(e) => {
                        println!("  ⚠ Could not detect specs: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            let server = Server {
                id: id.clone(),
                name: name.clone(),
                host: host.clone(),
                server_type: server_type.clone(),
                provider: provider.clone(),
                ssh_connection_id: ssh,
                location,
                specs,
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
            if let Some(specs) = &server.specs {
                println!();
                println!("  Specs:");
                if let Some(cpu) = specs.cpu_cores {
                    println!("    CPU:    {} cores", cpu);
                }
                if let Some(ram) = specs.ram_gb {
                    println!("    RAM:    {} GB", ram);
                }
                if let Some(disk) = specs.disk_gb {
                    println!("    Disk:   {} GB", disk);
                }
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

/// Detect server specs via SSH connection
async fn detect_specs_via_ssh(db: &Database, ssh_id: &str) -> anyhow::Result<ServerSpecs> {
    // Load SSH connection from database
    let ssh_conn = db
        .get_ssh_connection(ssh_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("SSH connection '{}' not found", ssh_id))?;

    // Create SSH manager and add connection
    let mut ssh_manager = SshManager::new();
    let conn_id = ssh_conn.id.clone();
    ssh_manager.add_connection(ssh_conn);

    // Detect specs (this is blocking, so we wrap in spawn_blocking)
    let specs =
        tokio::task::spawn_blocking(move || ssh_manager.detect_server_specs(&conn_id, None))
            .await??;

    Ok(specs)
}
