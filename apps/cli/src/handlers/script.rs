//! Script command handler

use crate::ScriptCommands;
use pctrl_core::{AuthMethod, Config, Script, ScriptType};
use pctrl_database::Database;
use pctrl_docker::DockerManager;
use pctrl_ssh::SshManager;

pub async fn handle(command: ScriptCommands, config: &Config, db: &Database) -> anyhow::Result<()> {
    match command {
        ScriptCommands::List => {
            let scripts = db.list_scripts().await?;
            if scripts.is_empty() {
                println!("No scripts configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl script add <name> -c <command> [-s server]");
                println!("  pctrl script add <name> -c <command> -t local");
                println!("  pctrl script add <name> -c <command> -t docker --docker-host <id> --container <id>");
            } else {
                println!("Scripts ({}):", scripts.len());
                println!();
                for script in scripts {
                    let danger_icon = if script.dangerous { "⚠️ " } else { "" };
                    println!(
                        "  📜 {}{} [{}]",
                        danger_icon, script.name, script.script_type
                    );
                }
            }
        }

        ScriptCommands::Add {
            name,
            command,
            description,
            script_type,
            server,
            project,
            docker_host,
            container,
            dangerous,
        } => {
            let id = name.to_lowercase().replace(' ', "-");

            let script_type: ScriptType = script_type.parse().unwrap_or_default();

            let script = Script {
                id: id.clone(),
                name: name.clone(),
                description,
                command: command.clone(),
                script_type: script_type.clone(),
                server_id: server.clone(),
                project_id: project,
                docker_host_id: docker_host.clone(),
                container_id: container.clone(),
                dangerous,
                last_run: None,
                last_result: None,
            };

            db.save_script(&script).await?;

            println!("✓ Script added:");
            println!();
            println!("  Name:    {}", name);
            println!("  ID:      {}", id);
            println!("  Type:    {}", script_type);
            println!("  Command: {}", command);
            if let Some(s) = server {
                println!("  Server:  {}", s);
            }
            if let Some(dh) = docker_host {
                println!("  Docker:  {}", dh);
            }
            if let Some(c) = container {
                println!("  Container: {}", c);
            }
            if dangerous {
                println!("  ⚠️  Marked as dangerous");
            }
        }

        ScriptCommands::Show { name } => {
            let script = db
                .get_script(&name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Script '{}' not found", name))?;

            let danger_icon = if script.dangerous { "⚠️ " } else { "" };

            println!();
            println!("  📜 {}{}", danger_icon, script.name);
            println!("  ─────────────────────────────");
            println!("  ID:      {}", script.id);
            println!("  Type:    {}", script.script_type);
            println!("  Command: {}", script.command);
            if let Some(desc) = &script.description {
                println!("  Desc:    {}", desc);
            }
            if let Some(server) = &script.server_id {
                println!("  Server:  {}", server);
            }
            if let Some(dh) = &script.docker_host_id {
                println!("  Docker:  {}", dh);
            }
            if let Some(c) = &script.container_id {
                println!("  Container: {}", c);
            }
            if let Some(project) = &script.project_id {
                println!("  Project: {}", project);
            }
            if let Some(last_run) = &script.last_run {
                println!("  Last Run: {}", last_run);
            }
            if let Some(result) = &script.last_result {
                println!("  Result:  {}", result);
            }
            println!();
        }

        ScriptCommands::Run { name, force } => {
            let script = db
                .get_script(&name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Script '{}' not found", name))?;

            if script.dangerous && !force {
                println!("⚠️  This script is marked as dangerous!");
                println!("    Command: {}", script.command);
                println!();
                println!("Use --force to run anyway.");
                return Ok(());
            }

            println!("Running script '{}'...", script.name);
            println!("Command: {}", script.command);
            println!();

            let result = match script.script_type {
                ScriptType::Local => execute_local(&script.command),

                ScriptType::Ssh => execute_ssh(&script, config, db).await?,

                ScriptType::Docker => execute_docker(&script, config).await?,
            };

            // Update script result in database
            db.update_script_result(&script.id, result).await?;
        }

        ScriptCommands::Remove { name } => {
            if db.remove_script(&name).await? {
                println!("✓ Script '{}' removed", name);
            } else {
                println!("✗ Script '{}' not found", name);
            }
        }
    }

    Ok(())
}

fn execute_local(command: &str) -> pctrl_core::ScriptResult {
    let shell = if cfg!(windows) { "cmd" } else { "sh" };
    let args = if cfg!(windows) {
        vec!["/C", command]
    } else {
        vec!["-c", command]
    };

    match std::process::Command::new(shell).args(&args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !stdout.is_empty() {
                println!("{}", stdout);
            }
            if !stderr.is_empty() {
                eprintln!("{}", stderr);
            }

            if output.status.success() {
                println!("✓ Script completed successfully");
                pctrl_core::ScriptResult::Success
            } else {
                println!(
                    "✗ Script failed with exit code: {}",
                    output.status.code().unwrap_or(-1)
                );
                pctrl_core::ScriptResult::Error
            }
        }
        Err(e) => {
            println!("✗ Failed to execute script: {}", e);
            pctrl_core::ScriptResult::Error
        }
    }
}

async fn execute_ssh(
    script: &Script,
    config: &Config,
    db: &Database,
) -> anyhow::Result<pctrl_core::ScriptResult> {
    let server_id = script.server_id.as_ref().ok_or_else(|| {
        anyhow::anyhow!("SSH script requires a server_id. Use -s <server> when adding.")
    })?;

    // Get server to find SSH connection
    let server = db
        .get_server(server_id)
        .await?
        .or(db.get_server_by_name(server_id).await?)
        .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", server_id))?;

    let ssh_id = server.ssh_connection_id.as_ref().ok_or_else(|| {
        anyhow::anyhow!("Server '{}' has no SSH connection configured", server.name)
    })?;

    // Initialize SSH manager
    let mut ssh_manager = SshManager::new();
    for conn in &config.ssh_connections {
        ssh_manager.add_connection(conn.clone());
    }

    // Check if password auth is needed
    let password = if let Some(conn) = ssh_manager.get_connection(ssh_id) {
        if matches!(conn.auth_method, AuthMethod::Password) {
            print!("Password for {}@{}: ", conn.username, conn.host);
            std::io::Write::flush(&mut std::io::stdout())?;
            Some(rpassword::read_password()?)
        } else {
            None
        }
    } else {
        None
    };

    match ssh_manager.execute_command_with_password(ssh_id, &script.command, password.as_deref()) {
        Ok(output) => {
            println!("{}", output);
            println!("✓ Script completed successfully");
            Ok(pctrl_core::ScriptResult::Success)
        }
        Err(e) => {
            println!("✗ SSH execution failed: {}", e);
            Ok(pctrl_core::ScriptResult::Error)
        }
    }
}

async fn execute_docker(
    script: &Script,
    config: &Config,
) -> anyhow::Result<pctrl_core::ScriptResult> {
    let docker_host_id = script.docker_host_id.as_ref().ok_or_else(|| {
        anyhow::anyhow!("Docker script requires docker_host_id. Use --docker-host when adding.")
    })?;

    let container_id = script.container_id.as_ref().ok_or_else(|| {
        anyhow::anyhow!("Docker script requires container_id. Use --container when adding.")
    })?;

    // Initialize Docker manager
    let mut docker_manager = DockerManager::new();
    for host in &config.docker_hosts {
        docker_manager.add_host(host.clone());
    }

    match docker_manager
        .exec_in_container(docker_host_id, container_id, &script.command)
        .await
    {
        Ok(output) => {
            println!("{}", output);
            println!("✓ Script completed successfully");
            Ok(pctrl_core::ScriptResult::Success)
        }
        Err(e) => {
            println!("✗ Docker execution failed: {}", e);
            Ok(pctrl_core::ScriptResult::Error)
        }
    }
}
