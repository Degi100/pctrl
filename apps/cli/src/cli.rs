use crate::{Commands, CoolifyCommands, DockerCommands, GitCommands, SshCommands};
use pctrl_coolify::CoolifyManager;
use pctrl_docker::DockerManager;
use pctrl_git::GitManager;
use pctrl_ssh::SshManager;

pub async fn handle_command(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Ssh { command } => handle_ssh_command(command).await,
        Commands::Docker { command } => handle_docker_command(command).await,
        Commands::Coolify { command } => handle_coolify_command(command).await,
        Commands::Git { command } => handle_git_command(command).await,
    }
}

async fn handle_ssh_command(command: SshCommands) -> anyhow::Result<()> {
    let ssh_manager = SshManager::new();

    match command {
        SshCommands::List => {
            let connections = ssh_manager.list_connections();
            println!("SSH Connections:");
            for conn in connections {
                println!(
                    "  [{}] {} - {}@{}:{}",
                    conn.id, conn.name, conn.username, conn.host, conn.port
                );
            }
        }
        SshCommands::Connect { id } => {
            println!("Connecting to SSH host: {}", id);
            let _session = ssh_manager.connect(&id)?;
            println!("Connected successfully");
        }
        SshCommands::Exec { id, command } => {
            println!("Executing command on {}: {}", id, command);
            let output = ssh_manager.execute_command(&id, &command)?;
            println!("Output:\n{}", output);
        }
    }

    Ok(())
}

async fn handle_docker_command(command: DockerCommands) -> anyhow::Result<()> {
    let docker_manager = DockerManager::new();

    match command {
        DockerCommands::List { host_id } => {
            let containers = docker_manager.list_containers(&host_id).await?;
            println!("Containers on host {}:", host_id);
            for container in containers {
                println!(
                    "  [{}] {} - {} ({})",
                    container.id, container.name, container.image, container.state
                );
            }
        }
        DockerCommands::Start {
            host_id,
            container_id,
        } => {
            docker_manager
                .start_container(&host_id, &container_id)
                .await?;
            println!("Container {} started", container_id);
        }
        DockerCommands::Stop {
            host_id,
            container_id,
        } => {
            docker_manager
                .stop_container(&host_id, &container_id)
                .await?;
            println!("Container {} stopped", container_id);
        }
    }

    Ok(())
}

async fn handle_coolify_command(command: CoolifyCommands) -> anyhow::Result<()> {
    let coolify_manager = CoolifyManager::new();

    match command {
        CoolifyCommands::List { instance_id } => {
            let deployments = coolify_manager.list_deployments(&instance_id).await?;
            println!("Deployments on instance {}:", instance_id);
            for deployment in deployments {
                println!(
                    "  [{}] {} - {}",
                    deployment.id, deployment.name, deployment.status
                );
            }
        }
        CoolifyCommands::Deploy {
            instance_id,
            project_id,
        } => {
            coolify_manager
                .deploy_project(&instance_id, &project_id)
                .await?;
            println!("Deployment started for project {}", project_id);
        }
    }

    Ok(())
}

async fn handle_git_command(command: GitCommands) -> anyhow::Result<()> {
    let git_manager = GitManager::new();

    match command {
        GitCommands::List { repo_id } => {
            let releases = git_manager.list_releases(&repo_id)?;
            println!("Releases in repository {}:", repo_id);
            for release in releases {
                println!("  [{}] {} - {}", release.tag, release.name, release.message);
            }
        }
        GitCommands::Create {
            repo_id,
            tag,
            message,
        } => {
            git_manager.create_release(&repo_id, &tag, &message)?;
            println!("Release {} created", tag);
        }
        GitCommands::Push { repo_id } => {
            git_manager.push_tags(&repo_id)?;
            println!("Tags pushed to remote");
        }
    }

    Ok(())
}
