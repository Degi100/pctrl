//! SSH command handler (Legacy - deprecated)

use crate::style;
use crate::SshCommands;
use pctrl_core::{AuthMethod, Config, SshConnection};
use pctrl_database::Database;
use pctrl_ssh::SshManager;

pub async fn handle(command: SshCommands, config: &Config, db: &Database) -> anyhow::Result<()> {
    style::deprecation_warning("ssh", "server");

    // Initialize manager with config data
    let mut ssh_manager = SshManager::new();
    for conn in &config.ssh_connections {
        ssh_manager.add_connection(conn.clone());
    }

    match command {
        SshCommands::List => {
            let connections = ssh_manager.list_connections();
            if connections.is_empty() {
                println!("No SSH connections configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl ssh add <name> <host> -u <user> [-k ~/.ssh/id_rsa]");
                println!("  pctrl ssh add <name> <host> -u <user> --password");
            } else {
                println!("SSH Connections ({}):", connections.len());
                println!();
                for conn in connections {
                    let auth_icon = match &conn.auth_method {
                        AuthMethod::PublicKey { .. } => "🔑",
                        AuthMethod::Password => "🔒",
                    };
                    println!(
                        "  {} [{}] {} - {}@{}:{}",
                        auth_icon, conn.id, conn.name, conn.username, conn.host, conn.port
                    );
                }
            }
        }

        SshCommands::Add {
            name,
            host,
            user,
            port,
            key,
            password,
        } => {
            let id = name.to_lowercase().replace(' ', "-");

            if db.ssh_connection_exists(&id).await? {
                anyhow::bail!("Connection '{}' already exists. Use a different name.", id);
            }

            let auth_method = if password {
                AuthMethod::Password
            } else {
                let key_path = key.unwrap_or_else(|| {
                    dirs::home_dir()
                        .map(|h| h.join(".ssh").join("id_rsa").to_string_lossy().to_string())
                        .unwrap_or_else(|| "~/.ssh/id_rsa".to_string())
                });
                AuthMethod::PublicKey { key_path }
            };

            let connection = SshConnection {
                id: id.clone(),
                name: name.clone(),
                host: host.clone(),
                port,
                username: user.clone(),
                auth_method: auth_method.clone(),
            };

            db.save_ssh_connection(&connection).await?;

            println!("✓ SSH connection added:");
            println!();
            println!("  Name:     {}", name);
            println!("  ID:       {}", id);
            println!("  Host:     {}:{}", host, port);
            println!("  User:     {}", user);
            match auth_method {
                AuthMethod::Password => println!("  Auth:     password"),
                AuthMethod::PublicKey { key_path } => println!("  Key:      {}", key_path),
            }
            println!();
            println!("Test with: pctrl ssh connect {}", id);
        }

        SshCommands::Remove { id } => {
            if db.remove_ssh_connection(&id).await? {
                println!("✓ SSH connection '{}' removed", id);
            } else {
                println!("✗ Connection '{}' not found", id);
            }
        }

        SshCommands::Connect { id } => {
            let password = prompt_password_if_needed(&ssh_manager, &id)?;

            println!("Connecting to SSH host: {}", id);
            let _session = ssh_manager.connect_with_password(&id, password.as_deref())?;
            println!("✓ Connected successfully");
        }

        SshCommands::Exec { id, command } => {
            let password = prompt_password_if_needed(&ssh_manager, &id)?;

            println!("Executing on {}: {}", id, command);
            let output =
                ssh_manager.execute_command_with_password(&id, &command, password.as_deref())?;
            println!("{}", output);
        }
    }

    Ok(())
}

fn prompt_password_if_needed(ssh_manager: &SshManager, id: &str) -> anyhow::Result<Option<String>> {
    if let Some(conn) = ssh_manager.get_connection(id) {
        if matches!(conn.auth_method, AuthMethod::Password) {
            print!("Password for {}@{}: ", conn.username, conn.host);
            std::io::Write::flush(&mut std::io::stdout())?;
            return Ok(Some(rpassword::read_password()?));
        }
    }
    Ok(None)
}
