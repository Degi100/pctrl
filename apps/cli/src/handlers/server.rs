//! Server command handler

use crate::ServerCommands;
use pctrl_core::{AuthMethod, CredentialData, Server, ServerSpecs, ServerType, SshConnection};
use pctrl_database::Database;
use pctrl_ssh::SshManager;

/// Server status information from SSH
#[derive(Default)]
struct ServerStatus {
    uptime: Option<String>,
    load: Option<String>,
    memory: Option<String>,
    disk: Option<String>,
}

pub async fn handle(command: ServerCommands, db: &Database) -> anyhow::Result<()> {
    match command {
        ServerCommands::List => {
            let servers = db.list_servers().await?;
            if servers.is_empty() {
                println!("No servers configured.");
                println!();
                println!("Add one with:");
                println!(
                    "  pctrl server add <name> <host> [-t type] [-p provider] [-c credential]"
                );
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
                    let cred_str = server
                        .credential_id
                        .as_ref()
                        .map(|c| format!(" [🔑 {}]", c))
                        .unwrap_or_default();
                    println!(
                        "  🖥️  {} - {} [{}]{}{}{}",
                        server.name,
                        server.host,
                        server.server_type,
                        provider_str,
                        specs_str,
                        cred_str
                    );
                }
            }
        }

        ServerCommands::Add {
            name,
            host,
            server_type,
            provider,
            credential,
            location,
        } => {
            let id = name.to_lowercase().replace(' ', "-");

            if db.get_server_by_name(&name).await?.is_some() {
                anyhow::bail!("Server '{}' already exists.", name);
            }

            let server_type: ServerType = server_type.parse().unwrap_or_default();

            // Resolve credential name to ID and auto-detect specs
            let (resolved_credential_id, specs): (Option<String>, Option<ServerSpecs>) =
                if let Some(ref cred_input) = credential {
                    // Look up credential by name or ID
                    let cred = db
                        .get_credential_by_name(cred_input)
                        .await?
                        .or(db.get_credential(cred_input).await?)
                        .ok_or_else(|| anyhow::anyhow!("Credential '{}' not found", cred_input))?;

                    let cred_id = cred.id.clone();

                    // Auto-detect specs via SSH
                    println!("🔍 Detecting server specs via SSH...");
                    let specs = match detect_specs_via_credential(db, &cred_id, &host).await {
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
                    };

                    (Some(cred_id), specs)
                } else {
                    (None, None)
                };

            let server = Server {
                id: id.clone(),
                name: name.clone(),
                host: host.clone(),
                server_type: server_type.clone(),
                provider: provider.clone(),
                credential_id: resolved_credential_id,
                location,
                specs,
                notes: None,
            };

            db.save_server(&server).await?;

            println!("✓ Server added:");
            println!();
            println!("  Name:       {}", name);
            println!("  ID:         {}", id);
            println!("  Host:       {}", host);
            println!("  Type:       {}", server_type);
            if let Some(p) = provider {
                println!("  Provider:   {}", p);
            }
            if let Some(c) = credential {
                println!("  Credential: {}", c);
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
            println!("  ID:         {}", server.id);
            println!("  Host:       {}", server.host);
            println!("  Type:       {}", server.server_type);
            if let Some(p) = &server.provider {
                println!("  Provider:   {}", p);
            }
            if let Some(l) = &server.location {
                println!("  Location:   {}", l);
            }
            if let Some(cred) = &server.credential_id {
                println!("  Credential: {}", cred);
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

        ServerCommands::Exec { name, command } => {
            let server = db
                .get_server_by_name(&name)
                .await?
                .or(db.get_server(&name).await?)
                .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", name))?;

            let cred_id = server
                .credential_id
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Server '{}' has no credential configured", name))?;

            println!("🔌 Connecting to {}...", server.host);
            let (ssh_manager, conn_id) = create_ssh_manager(db, cred_id, &server.host).await?;

            println!("▶ Executing: {}", command);
            println!();

            let output = tokio::task::spawn_blocking(move || {
                ssh_manager.execute_command(&conn_id, &command)
            })
            .await??;

            // Print output
            print!("{}", output);
        }

        ServerCommands::Status { name } => {
            let server = db
                .get_server_by_name(&name)
                .await?
                .or(db.get_server(&name).await?)
                .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", name))?;

            println!();
            println!("  🖥️  {} ({})", server.name, server.host);
            println!("  ─────────────────────────────");

            // Check if credential is configured
            let Some(cred_id) = &server.credential_id else {
                println!("  ⚠ No credential configured");
                println!();
                return Ok(());
            };

            // Try to connect and get status
            print!("  Connecting... ");
            match create_ssh_manager(db, cred_id, &server.host).await {
                Ok((ssh_manager, conn_id)) => {
                    // Run all status commands in a single blocking task
                    let status_result = tokio::task::spawn_blocking(move || {
                        let mut results = ServerStatus::default();

                        // Get uptime
                        if let Ok(output) =
                            ssh_manager.execute_command(&conn_id, "uptime -p 2>/dev/null || uptime")
                        {
                            results.uptime = Some(output.trim().to_string());
                        }

                        // Get load average
                        if let Ok(output) = ssh_manager
                            .execute_command(&conn_id, "cat /proc/loadavg | cut -d' ' -f1-3")
                        {
                            results.load = Some(output.trim().to_string());
                        }

                        // Get memory info
                        if let Ok(output) = ssh_manager.execute_command(
                            &conn_id,
                            "free -h | grep Mem | awk '{print $3 \"/\" $2}'",
                        ) {
                            results.memory = Some(output.trim().to_string());
                        }

                        // Get disk usage
                        if let Ok(output) = ssh_manager.execute_command(
                            &conn_id,
                            "df -h / | tail -1 | awk '{print $3 \"/\" $2 \" (\" $5 \")\"}'",
                        ) {
                            results.disk = Some(output.trim().to_string());
                        }

                        results
                    })
                    .await?;

                    println!("✓");

                    if let Some(uptime) = &status_result.uptime {
                        if !uptime.is_empty() {
                            println!("  Uptime:  {}", uptime);
                        }
                    }
                    if let Some(load) = &status_result.load {
                        if !load.is_empty() {
                            println!("  Load:    {}", load);
                        }
                    }
                    if let Some(memory) = &status_result.memory {
                        if !memory.is_empty() {
                            println!("  Memory:  {}", memory);
                        }
                    }
                    if let Some(disk) = &status_result.disk {
                        if !disk.is_empty() {
                            println!("  Disk:    {}", disk);
                        }
                    }

                    println!();
                    println!("  Status:  ✓ Online");
                }
                Err(e) => {
                    println!("✗");
                    println!("  Status:  ✗ Offline ({})", e);
                }
            }
            println!();
        }
    }

    Ok(())
}

/// Create SSH manager from credential
async fn create_ssh_manager(
    db: &Database,
    cred_id: &str,
    host: &str,
) -> anyhow::Result<(SshManager, String)> {
    // Load credential from database (by name or ID)
    let credential = db
        .get_credential_by_name(cred_id)
        .await?
        .or(db.get_credential(cred_id).await?)
        .ok_or_else(|| anyhow::anyhow!("Credential '{}' not found", cred_id))?;

    // Extract SSH details from credential and create appropriate auth method
    let (username, port, auth_method) = match &credential.data {
        CredentialData::SshKey {
            username,
            port,
            key_path,
            passphrase,
        } => (
            username.clone(),
            *port,
            AuthMethod::Key {
                path: key_path.clone(),
                passphrase: passphrase.clone(),
            },
        ),
        CredentialData::SshAgent { username, port } => (username.clone(), *port, AuthMethod::Agent),
        _ => anyhow::bail!("Credential '{}' is not an SSH credential", cred_id),
    };

    // Create SSH connection for pctrl-ssh
    let ssh_conn = SshConnection {
        id: credential.id.clone(),
        name: credential.name.clone(),
        host: host.to_string(),
        port,
        username,
        auth_method,
    };

    // Create SSH manager and add connection
    let mut ssh_manager = SshManager::new();
    let conn_id = ssh_conn.id.clone();
    ssh_manager.add_connection(ssh_conn);

    Ok((ssh_manager, conn_id))
}

/// Detect server specs via SSH credential
async fn detect_specs_via_credential(
    db: &Database,
    cred_id: &str,
    host: &str,
) -> anyhow::Result<ServerSpecs> {
    let (ssh_manager, conn_id) = create_ssh_manager(db, cred_id, host).await?;

    // Detect specs (this is blocking, so we wrap in spawn_blocking)
    let specs =
        tokio::task::spawn_blocking(move || ssh_manager.detect_server_specs(&conn_id, None))
            .await??;

    Ok(specs)
}
