//! Coolify command handler

use crate::CoolifyCommands;
use pctrl_coolify::CoolifyManager;
use pctrl_core::{Config, CoolifyInstance};
use pctrl_database::Database;

pub async fn handle(
    command: CoolifyCommands,
    config: &Config,
    db: &Database,
) -> anyhow::Result<()> {
    // Initialize manager with config data
    let mut coolify_manager = CoolifyManager::new();
    for instance in &config.coolify_instances {
        coolify_manager.add_instance(instance.clone());
    }

    match command {
        CoolifyCommands::Instances => {
            if config.coolify_instances.is_empty() {
                println!("No Coolify instances configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl coolify add <name> -u <url> -t <token>");
            } else {
                println!("Coolify Instances ({}):", config.coolify_instances.len());
                println!();
                for instance in &config.coolify_instances {
                    println!(
                        "  🚀 [{}] {} - {}",
                        instance.id, instance.name, instance.url
                    );
                }
            }
        }

        CoolifyCommands::Add { name, url, token } => {
            let id = name.to_lowercase().replace(' ', "-");

            if db.coolify_instance_exists(&id).await? {
                anyhow::bail!(
                    "Coolify instance '{}' already exists. Use a different name.",
                    id
                );
            }

            let instance = CoolifyInstance {
                id: id.clone(),
                name: name.clone(),
                url: url.clone(),
                api_key: token,
            };

            db.save_coolify_instance(&instance).await?;

            println!("✓ Coolify instance added:");
            println!();
            println!("  Name:  {}", name);
            println!("  ID:    {}", id);
            println!("  URL:   {}", url);
            println!();
            println!("List deployments with: pctrl coolify list {}", id);
        }

        CoolifyCommands::Remove { id } => {
            if db.remove_coolify_instance(&id).await? {
                println!("✓ Coolify instance '{}' removed", id);
            } else {
                println!("✗ Coolify instance '{}' not found", id);
            }
        }

        CoolifyCommands::List { instance_id } => {
            let deployments = coolify_manager.list_deployments(&instance_id).await?;
            if deployments.is_empty() {
                println!("No deployments on instance {}", instance_id);
            } else {
                println!("Deployments on {} ({}):", instance_id, deployments.len());
                println!();
                for deployment in deployments {
                    let status_icon = match deployment.status.as_str() {
                        "running" | "healthy" => "●",
                        "stopped" | "exited" => "○",
                        "error" | "failed" => "✗",
                        _ => "◌",
                    };
                    println!(
                        "  {} [{}] {} - {}",
                        status_icon, deployment.id, deployment.name, deployment.status
                    );
                }
            }
        }

        CoolifyCommands::Deploy {
            instance_id,
            project_id,
        } => {
            coolify_manager
                .deploy_project(&instance_id, &project_id)
                .await?;
            println!("✓ Deployment started for project {}", project_id);
        }
    }

    Ok(())
}
