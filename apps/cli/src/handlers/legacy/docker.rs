//! Docker command handler

use crate::DockerCommands;
use pctrl_core::{Config, DockerHost};
use pctrl_database::Database;
use pctrl_docker::DockerManager;

pub async fn handle(command: DockerCommands, config: &Config, db: &Database) -> anyhow::Result<()> {
    // Initialize manager with config data
    let mut docker_manager = DockerManager::new();
    for host in &config.docker_hosts {
        docker_manager.add_host(host.clone());
    }

    match command {
        DockerCommands::Hosts => {
            if config.docker_hosts.is_empty() {
                println!("No Docker hosts configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl docker add <name> [-u <url>]");
            } else {
                println!("Docker Hosts ({}):", config.docker_hosts.len());
                println!();
                for host in &config.docker_hosts {
                    println!("  🐳 [{}] {} - {}", host.id, host.name, host.url);
                }
            }
        }

        DockerCommands::Add { name, url } => {
            let id = name.to_lowercase().replace(' ', "-");

            if db.docker_host_exists(&id).await? {
                anyhow::bail!("Docker host '{}' already exists. Use a different name.", id);
            }

            let host = DockerHost {
                id: id.clone(),
                name: name.clone(),
                url: url.clone(),
            };

            db.save_docker_host(&host).await?;

            println!("✓ Docker host added:");
            println!();
            println!("  Name:  {}", name);
            println!("  ID:    {}", id);
            println!("  URL:   {}", url);
            println!();
            println!("List containers with: pctrl docker list {}", id);
        }

        DockerCommands::Remove { id } => {
            if db.remove_docker_host(&id).await? {
                println!("✓ Docker host '{}' removed", id);
            } else {
                println!("✗ Docker host '{}' not found", id);
            }
        }

        DockerCommands::List { host_id } => {
            let containers = docker_manager.list_containers(&host_id).await?;
            if containers.is_empty() {
                println!("No containers on host {}", host_id);
            } else {
                println!("Containers on {} ({}):", host_id, containers.len());
                println!();
                for container in containers {
                    let state_icon = match container.state.as_str() {
                        "running" => "●",
                        "exited" => "○",
                        _ => "◌",
                    };
                    println!(
                        "  {} [{}] {} - {}",
                        state_icon,
                        &container.id[..12],
                        container.name,
                        container.image
                    );
                }
            }
        }

        DockerCommands::Start {
            host_id,
            container_id,
        } => {
            docker_manager
                .start_container(&host_id, &container_id)
                .await?;
            println!("✓ Container {} started", container_id);
        }

        DockerCommands::Stop {
            host_id,
            container_id,
        } => {
            docker_manager
                .stop_container(&host_id, &container_id)
                .await?;
            println!("✓ Container {} stopped", container_id);
        }
    }

    Ok(())
}
