//! Migration handler - migrate legacy data to v6 structure

use crate::style;
use pctrl_core::{Config, ProjectResource, ResourceType, Server, ServerType};
use pctrl_database::Database;
use std::io::{self, Write};

pub async fn handle(
    auto: bool,
    cleanup: bool,
    config: &Config,
    db: &Database,
) -> anyhow::Result<()> {
    println!();
    println!(
        "{}┌─────────────────────────────────────────┐{}",
        style::CYAN,
        style::RESET
    );
    println!(
        "{}│{}  {}pctrl migrate{} - Legacy → v6            {}│{}",
        style::CYAN,
        style::RESET,
        style::BOLD,
        style::RESET,
        style::CYAN,
        style::RESET
    );
    println!(
        "{}└─────────────────────────────────────────┘{}",
        style::CYAN,
        style::RESET
    );
    println!();

    // Scan legacy data
    println!("{}Scanning legacy data...{}", style::DIM, style::RESET);

    let ssh_count = config.ssh_connections.len();
    let docker_count = config.docker_hosts.len();
    let coolify_count = config.coolify_instances.len();
    let git_count = config.git_repos.len();

    println!(
        "  Found {}{}{} SSH Connections",
        style::BOLD,
        ssh_count,
        style::RESET
    );
    println!(
        "  Found {}{}{} Docker Hosts",
        style::BOLD,
        docker_count,
        style::RESET
    );
    println!(
        "  Found {}{}{} Coolify Instances",
        style::BOLD,
        coolify_count,
        style::RESET
    );
    println!(
        "  Found {}{}{} Git Repos",
        style::BOLD,
        git_count,
        style::RESET
    );
    println!();

    let total = ssh_count + docker_count + coolify_count + git_count;
    if total == 0 {
        println!("{}No legacy data to migrate.{}", style::GREEN, style::RESET);
        return Ok(());
    }

    // Get available projects for linking
    let projects = db.list_projects().await?;
    let project_names: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();

    println!(
        "{}─────────────────────────────────────────{}",
        style::GRAY,
        style::RESET
    );
    println!();

    let mut stats = MigrationStats::default();

    // Migrate SSH Connections → Servers
    for (i, ssh) in config.ssh_connections.iter().enumerate() {
        println!(
            "{}[{}/{}]{} SSH Connection: {}{}{} ({}@{}:{})",
            style::CYAN,
            i + 1,
            ssh_count,
            style::RESET,
            style::BOLD,
            ssh.name,
            style::RESET,
            ssh.username,
            ssh.host,
            ssh.port
        );
        println!();

        // Check if server already exists
        if db.get_server_by_name(&ssh.name).await?.is_some() {
            println!(
                "  {}○{} Server '{}' already exists, skipping",
                style::YELLOW,
                style::RESET,
                ssh.name
            );
            stats.skipped += 1;
            println!();
            continue;
        }

        // Ask for confirmation
        let create = if auto {
            true
        } else {
            prompt_yes_no(&format!("  → Create Server '{}'?", ssh.name), true)?
        };

        if !create {
            println!("  {}○{} Skipped", style::YELLOW, style::RESET);
            stats.skipped += 1;
            println!();
            continue;
        }

        // Create server from SSH connection
        let server = Server {
            id: ssh.id.clone(),
            name: ssh.name.clone(),
            host: ssh.host.clone(),
            server_type: ServerType::Vps,
            provider: None,
            ssh_connection_id: Some(ssh.id.clone()),
            location: None,
            specs: None,
            notes: Some(format!("Migrated from SSH connection '{}'", ssh.id)),
        };

        db.save_server(&server).await?;
        println!(
            "  {}✓{} Server '{}' created",
            style::GREEN,
            style::RESET,
            server.name
        );
        stats.servers_created += 1;

        // Ask for project linking
        if !projects.is_empty() {
            let project_choice = if auto {
                None
            } else {
                prompt_choice("  → Link to project?", &project_names, true)?
            };

            if let Some(project_name) = project_choice {
                if let Some(project) = projects.iter().find(|p| p.name == project_name) {
                    let resource = ProjectResource {
                        id: uuid::Uuid::new_v4().to_string(),
                        project_id: project.id.clone(),
                        resource_type: ResourceType::Server,
                        resource_id: server.id.clone(),
                        role: Some("server".to_string()),
                        notes: None,
                    };
                    db.link_project_resource(&resource).await?;
                    println!(
                        "  {}✓{} Linked to project '{}'",
                        style::GREEN,
                        style::RESET,
                        project_name
                    );
                    stats.links_created += 1;
                }
            }
        }

        println!();
    }

    // Migrate Docker Hosts → Project Resources (keep as docker_hosts, just link)
    for (i, docker) in config.docker_hosts.iter().enumerate() {
        println!(
            "{}[{}/{}]{} Docker Host: {}{}{} ({})",
            style::CYAN,
            i + 1,
            docker_count,
            style::RESET,
            style::BOLD,
            docker.name,
            style::RESET,
            docker.url
        );
        println!();

        if projects.is_empty() {
            println!(
                "  {}○{} No projects to link to, skipping",
                style::YELLOW,
                style::RESET
            );
            stats.skipped += 1;
            println!();
            continue;
        }

        let project_choice = if auto {
            None
        } else {
            prompt_choice("  → Link to project?", &project_names, true)?
        };

        if let Some(project_name) = project_choice {
            if let Some(project) = projects.iter().find(|p| p.name == project_name) {
                let resource = ProjectResource {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: project.id.clone(),
                    resource_type: ResourceType::Container,
                    resource_id: docker.id.clone(),
                    role: Some("docker-host".to_string()),
                    notes: None,
                };
                db.link_project_resource(&resource).await?;
                println!(
                    "  {}✓{} Linked to project '{}'",
                    style::GREEN,
                    style::RESET,
                    project_name
                );
                stats.links_created += 1;
            }
        } else {
            stats.skipped += 1;
        }

        println!();
    }

    // Migrate Coolify Instances → Project Resources
    for (i, coolify) in config.coolify_instances.iter().enumerate() {
        println!(
            "{}[{}/{}]{} Coolify Instance: {}{}{} ({})",
            style::CYAN,
            i + 1,
            coolify_count,
            style::RESET,
            style::BOLD,
            coolify.name,
            style::RESET,
            coolify.url
        );
        println!();

        if projects.is_empty() {
            println!(
                "  {}○{} No projects to link to, skipping",
                style::YELLOW,
                style::RESET
            );
            stats.skipped += 1;
            println!();
            continue;
        }

        let project_choice = if auto {
            None
        } else {
            prompt_choice("  → Link to project?", &project_names, true)?
        };

        if let Some(project_name) = project_choice {
            if let Some(project) = projects.iter().find(|p| p.name == project_name) {
                let resource = ProjectResource {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: project.id.clone(),
                    resource_type: ResourceType::Coolify,
                    resource_id: coolify.id.clone(),
                    role: Some("deployment".to_string()),
                    notes: None,
                };
                db.link_project_resource(&resource).await?;
                println!(
                    "  {}✓{} Linked to project '{}'",
                    style::GREEN,
                    style::RESET,
                    project_name
                );
                stats.links_created += 1;
            }
        } else {
            stats.skipped += 1;
        }

        println!();
    }

    // Migrate Git Repos → Project Resources
    for (i, git) in config.git_repos.iter().enumerate() {
        println!(
            "{}[{}/{}]{} Git Repo: {}{}{} ({})",
            style::CYAN,
            i + 1,
            git_count,
            style::RESET,
            style::BOLD,
            git.name,
            style::RESET,
            git.path
        );
        println!();

        if projects.is_empty() {
            println!(
                "  {}○{} No projects to link to, skipping",
                style::YELLOW,
                style::RESET
            );
            stats.skipped += 1;
            println!();
            continue;
        }

        let project_choice = if auto {
            None
        } else {
            prompt_choice("  → Link to project?", &project_names, true)?
        };

        if let Some(project_name) = project_choice {
            if let Some(project) = projects.iter().find(|p| p.name == project_name) {
                let resource = ProjectResource {
                    id: uuid::Uuid::new_v4().to_string(),
                    project_id: project.id.clone(),
                    resource_type: ResourceType::Git,
                    resource_id: git.id.clone(),
                    role: Some("repository".to_string()),
                    notes: None,
                };
                db.link_project_resource(&resource).await?;
                println!(
                    "  {}✓{} Linked to project '{}'",
                    style::GREEN,
                    style::RESET,
                    project_name
                );
                stats.links_created += 1;
            }
        } else {
            stats.skipped += 1;
        }

        println!();
    }

    // Summary
    println!(
        "{}─────────────────────────────────────────{}",
        style::GRAY,
        style::RESET
    );
    println!();
    println!("{}Summary:{}", style::BOLD, style::RESET);
    println!(
        "  {}✓{} {} Servers created",
        style::GREEN,
        style::RESET,
        stats.servers_created
    );
    println!(
        "  {}✓{} {} Project links created",
        style::GREEN,
        style::RESET,
        stats.links_created
    );
    println!(
        "  {}○{} {} Skipped",
        style::YELLOW,
        style::RESET,
        stats.skipped
    );
    println!();

    if cleanup {
        println!(
            "{}⚠{} Cleanup flag detected - legacy data removal not yet implemented",
            style::YELLOW,
            style::RESET
        );
        println!("  Legacy data preserved for safety.");
    } else {
        println!("{}Legacy data preserved.{}", style::DIM, style::RESET);
        println!(
            "{}Use 'pctrl migrate --cleanup' to remove after verification.{}",
            style::DIM,
            style::RESET
        );
    }
    println!();

    Ok(())
}

#[derive(Default)]
struct MigrationStats {
    servers_created: usize,
    links_created: usize,
    skipped: usize,
}

/// Prompt for yes/no with default
fn prompt_yes_no(question: &str, default_yes: bool) -> io::Result<bool> {
    let hint = if default_yes { "[Y/n]" } else { "[y/N]" };
    print!("{} {} ", question, hint);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    Ok(match input.as_str() {
        "" => default_yes,
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default_yes,
    })
}

/// Prompt for choice from list, with "none" option
fn prompt_choice(question: &str, options: &[&str], allow_none: bool) -> io::Result<Option<String>> {
    println!("{}", question);
    for (i, opt) in options.iter().enumerate() {
        println!("    {}[{}]{} {}", style::CYAN, i + 1, style::RESET, opt);
    }
    if allow_none {
        println!("    {}[n]{} none", style::DIM, style::RESET);
    }
    print!("  Choice: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input == "n" || input == "none" || input.is_empty() {
        return Ok(None);
    }

    if let Ok(idx) = input.parse::<usize>() {
        if idx > 0 && idx <= options.len() {
            return Ok(Some(options[idx - 1].to_string()));
        }
    }

    // Try to match by name
    for opt in options {
        if opt.to_lowercase().starts_with(&input) {
            return Ok(Some(opt.to_string()));
        }
    }

    Ok(None)
}
