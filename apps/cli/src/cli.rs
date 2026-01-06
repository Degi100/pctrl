use crate::{Commands, CoolifyCommands, DockerCommands, GitCommands, SshCommands};
use pctrl_coolify::CoolifyManager;
use pctrl_core::{AuthMethod, Config, CoolifyInstance, DockerHost, GitRepo, SshConnection};
use pctrl_database::Database;
use pctrl_docker::DockerManager;
use pctrl_git::GitManager;
use pctrl_ssh::SshManager;
use std::sync::Arc;

pub async fn handle_command(
    command: Commands,
    config: Arc<Config>,
    db: Arc<Database>,
) -> anyhow::Result<()> {
    match command {
        Commands::Ssh { command } => handle_ssh_command(command, &config, &db).await,
        Commands::Docker { command } => handle_docker_command(command, &config, &db).await,
        Commands::Coolify { command } => handle_coolify_command(command, &config, &db).await,
        Commands::Git { command } => handle_git_command(command, &config, &db).await,
    }
}

async fn handle_ssh_command(
    command: SshCommands,
    config: &Config,
    db: &Database,
) -> anyhow::Result<()> {
    // ─────────────────────────────────────────────────────────────────────────
    // Manager mit Config-Daten initialisieren
    // ─────────────────────────────────────────────────────────────────────────
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
        } => {
            // ID = name (lowercase, keine Leerzeichen)
            let id = name.to_lowercase().replace(' ', "-");

            // Prüfen ob schon existiert
            if db.ssh_connection_exists(&id).await? {
                anyhow::bail!("Connection '{}' already exists. Use a different name.", id);
            }

            // Default Key-Pfad
            let key_path = key.unwrap_or_else(|| {
                dirs::home_dir()
                    .map(|h| h.join(".ssh").join("id_rsa").to_string_lossy().to_string())
                    .unwrap_or_else(|| "~/.ssh/id_rsa".to_string())
            });

            let connection = SshConnection {
                id: id.clone(),
                name: name.clone(),
                host: host.clone(),
                port,
                username: user.clone(),
                auth_method: AuthMethod::PublicKey { key_path },
            };

            // In DB speichern
            db.save_ssh_connection(&connection).await?;

            println!("✓ SSH connection added:");
            println!();
            println!("  Name:     {}", name);
            println!("  ID:       {}", id);
            println!("  Host:     {}:{}", host, port);
            println!("  User:     {}", user);
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
            println!("Connecting to SSH host: {}", id);
            let _session = ssh_manager.connect(&id)?;
            println!("✓ Connected successfully");
        }

        SshCommands::Exec { id, command } => {
            println!("Executing on {}: {}", id, command);
            let output = ssh_manager.execute_command(&id, &command)?;
            println!("{}", output);
        }
    }

    Ok(())
}

async fn handle_docker_command(
    command: DockerCommands,
    config: &Config,
    db: &Database,
) -> anyhow::Result<()> {
    // ─────────────────────────────────────────────────────────────────────────
    // Manager mit Config-Daten initialisieren
    // ─────────────────────────────────────────────────────────────────────────
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
                        container.id[..12].to_string(),
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

async fn handle_coolify_command(
    command: CoolifyCommands,
    config: &Config,
    db: &Database,
) -> anyhow::Result<()> {
    // ─────────────────────────────────────────────────────────────────────────
    // Manager mit Config-Daten initialisieren
    // ─────────────────────────────────────────────────────────────────────────
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
                    println!("  🚀 [{}] {} - {}", instance.id, instance.name, instance.url);
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

async fn handle_git_command(
    command: GitCommands,
    config: &Config,
    db: &Database,
) -> anyhow::Result<()> {
    // ─────────────────────────────────────────────────────────────────────────
    // Manager mit Config-Daten initialisieren
    // ─────────────────────────────────────────────────────────────────────────
    let mut git_manager = GitManager::new();
    for repo in &config.git_repos {
        git_manager.add_repo(repo.clone());
    }

    match command {
        GitCommands::Repos => {
            if config.git_repos.is_empty() {
                println!("No Git repositories configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl git add <name> -p <path>");
            } else {
                println!("Git Repositories ({}):", config.git_repos.len());
                println!();
                for repo in &config.git_repos {
                    println!("  📁 [{}] {} - {}", repo.id, repo.name, repo.path);
                }
            }
        }

        GitCommands::Add { name, path } => {
            let id = name.to_lowercase().replace(' ', "-");

            if db.git_repo_exists(&id).await? {
                anyhow::bail!("Git repository '{}' already exists. Use a different name.", id);
            }

            // Verify path exists
            let abs_path = std::path::Path::new(&path);
            if !abs_path.exists() {
                anyhow::bail!("Path '{}' does not exist.", path);
            }

            let repo = GitRepo {
                id: id.clone(),
                name: name.clone(),
                path: path.clone(),
                remote_url: None,
            };

            db.save_git_repo(&repo).await?;

            println!("✓ Git repository added:");
            println!();
            println!("  Name:  {}", name);
            println!("  ID:    {}", id);
            println!("  Path:  {}", path);
            println!();
            println!("List releases with: pctrl git list {}", id);
        }

        GitCommands::Remove { id } => {
            if db.remove_git_repo(&id).await? {
                println!("✓ Git repository '{}' removed", id);
            } else {
                println!("✗ Git repository '{}' not found", id);
            }
        }

        GitCommands::List { repo_id } => {
            let releases = git_manager.list_releases(&repo_id)?;
            if releases.is_empty() {
                println!("No releases in repository {}", repo_id);
            } else {
                println!("Releases in {} ({}):", repo_id, releases.len());
                println!();
                for release in releases {
                    println!("  [{}] {} - {}", release.tag, release.name, release.message);
                }
            }
        }
        GitCommands::Create {
            repo_id,
            tag,
            message,
        } => {
            git_manager.create_release(&repo_id, &tag, &message)?;
            println!("✓ Release {} created", tag);
        }
        GitCommands::Push { repo_id } => {
            git_manager.push_tags(&repo_id)?;
            println!("✓ Tags pushed to remote");
        }
    }

    Ok(())
}
