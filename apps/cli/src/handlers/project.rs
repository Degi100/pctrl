//! Project command handler

use crate::ProjectCommands;
use pctrl_core::{Project, ProjectResource, ProjectStatus, ResourceType};
use pctrl_database::Database;

pub async fn handle(command: ProjectCommands, db: &Database) -> anyhow::Result<()> {
    match command {
        ProjectCommands::List => {
            let projects = db.list_projects().await?;
            if projects.is_empty() {
                println!("No projects configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl project add <name> [-d description] [-s stack]");
            } else {
                println!("Projects ({}):", projects.len());
                println!();
                for project in projects {
                    let status_icon = match project.status {
                        ProjectStatus::Live => "🟢",
                        ProjectStatus::Staging => "🟡",
                        ProjectStatus::Dev => "🔵",
                        ProjectStatus::Archived => "⚫",
                    };
                    let stack_str = if project.stack.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", project.stack.join(", "))
                    };
                    println!(
                        "  {} {} - {}{}",
                        status_icon, project.name, project.status, stack_str
                    );
                }
            }
        }

        ProjectCommands::Add {
            name,
            description,
            stack,
            status,
        } => {
            let id = name.to_lowercase().replace(' ', "-");

            if db.get_project_by_name(&name).await?.is_some() {
                anyhow::bail!("Project '{}' already exists.", name);
            }

            let stack_vec: Vec<String> = stack
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let status: ProjectStatus = status.parse().unwrap_or_default();

            let project = Project {
                id: id.clone(),
                name: name.clone(),
                description,
                stack: stack_vec.clone(),
                status: status.clone(),
                color: None,
                icon: None,
                notes: None,
            };

            db.save_project(&project).await?;

            println!("✓ Project added:");
            println!();
            println!("  Name:   {}", name);
            println!("  ID:     {}", id);
            println!("  Status: {}", status);
            if !stack_vec.is_empty() {
                println!("  Stack:  {}", stack_vec.join(", "));
            }
        }

        ProjectCommands::Show { name } => {
            let project = db
                .get_project_by_name(&name)
                .await?
                .or(db.get_project(&name).await?)
                .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", name))?;

            let status_icon = match project.status {
                ProjectStatus::Live => "🟢",
                ProjectStatus::Staging => "🟡",
                ProjectStatus::Dev => "🔵",
                ProjectStatus::Archived => "⚫",
            };

            println!();
            println!("  {} {}", status_icon, project.name);
            println!("  ─────────────────────────────");
            println!("  ID:     {}", project.id);
            println!("  Status: {}", project.status);
            if !project.stack.is_empty() {
                println!("  Stack:  {}", project.stack.join(", "));
            }
            if let Some(desc) = &project.description {
                println!("  Desc:   {}", desc);
            }

            // Show linked resources
            let resources = db.get_project_resources(&project.id).await?;
            if !resources.is_empty() {
                println!();
                println!("  Resources ({}):", resources.len());
                for res in resources {
                    let role_str = res.role.map(|r| format!(" ({})", r)).unwrap_or_default();
                    println!(
                        "    {} {} → {}{}",
                        res.resource_type, res.resource_id, res.id, role_str
                    );
                }
            }
            println!();
        }

        ProjectCommands::Remove { name } => {
            let project = db
                .get_project_by_name(&name)
                .await?
                .or(db.get_project(&name).await?)
                .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", name))?;

            if db.remove_project(&project.id).await? {
                println!("✓ Project '{}' removed", project.name);
            }
        }

        ProjectCommands::Link {
            project,
            resource_type,
            resource_id,
            role,
        } => {
            let proj = db
                .get_project_by_name(&project)
                .await?
                .or(db.get_project(&project).await?)
                .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project))?;

            let res_type: ResourceType = resource_type
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;

            let link = ProjectResource {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: proj.id.clone(),
                resource_type: res_type.clone(),
                resource_id: resource_id.clone(),
                role: role.clone(),
                notes: None,
            };

            db.link_project_resource(&link).await?;

            println!(
                "✓ Linked {} '{}' to project '{}'",
                res_type, resource_id, proj.name
            );
            if let Some(r) = role {
                println!("  Role: {}", r);
            }
        }

        ProjectCommands::Unlink { project, link_id } => {
            let proj = db
                .get_project_by_name(&project)
                .await?
                .or(db.get_project(&project).await?)
                .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project))?;

            if db.unlink_project_resource(&link_id).await? {
                println!("✓ Unlinked resource from project '{}'", proj.name);
            } else {
                println!("✗ Link '{}' not found", link_id);
            }
        }
    }

    Ok(())
}
