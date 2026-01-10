//! Script command handler

use crate::ScriptCommands;
use pctrl_core::{Script, ScriptType};
use pctrl_database::Database;

pub async fn handle(command: ScriptCommands, db: &Database) -> anyhow::Result<()> {
    match command {
        ScriptCommands::List => {
            let scripts = db.list_scripts().await?;
            if scripts.is_empty() {
                println!("No scripts configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl script add <name> -c <command>");
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
                exit_code: None,
                last_output: None,
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
                let exit_info = script
                    .exit_code
                    .map(|c| format!(" (exit {})", c))
                    .unwrap_or_default();
                println!("  Result:  {}{}", result, exit_info);
            }
            if let Some(output) = &script.last_output {
                if !output.is_empty() {
                    println!("  Output:");
                    for line in output.lines().take(10) {
                        println!("    {}", line);
                    }
                    if output.lines().count() > 10 {
                        println!("    ... ({} more lines)", output.lines().count() - 10);
                    }
                }
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

            let (result, exit_code, output) = match script.script_type {
                ScriptType::Local => execute_local(&script.command),
                ScriptType::Ssh => {
                    println!("⚠️  SSH script execution not yet implemented in v6.");
                    println!("    Use local scripts for now.");
                    return Ok(());
                }
                ScriptType::Docker => {
                    println!("⚠️  Docker script execution not yet implemented in v6.");
                    println!("    Use local scripts for now.");
                    return Ok(());
                }
            };

            // Update script result in database
            db.update_script_result(&script.id, result, exit_code, output.as_deref())
                .await?;
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

/// Result type for script execution: (result, exit_code, output)
type ExecResult = (pctrl_core::ScriptResult, Option<i32>, Option<String>);

fn execute_local(command: &str) -> ExecResult {
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

            let combined_output = format!("{}{}", stdout, stderr);
            let exit_code = output.status.code();

            if output.status.success() {
                println!("✓ Script completed successfully");
                (
                    pctrl_core::ScriptResult::Success,
                    exit_code,
                    Some(combined_output),
                )
            } else {
                println!(
                    "✗ Script failed with exit code: {}",
                    exit_code.unwrap_or(-1)
                );
                (
                    pctrl_core::ScriptResult::Error,
                    exit_code,
                    Some(combined_output),
                )
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to execute: {}", e);
            println!("✗ {}", error_msg);
            (pctrl_core::ScriptResult::Error, None, Some(error_msg))
        }
    }
}
