//! Git command handler (Legacy - deprecated)

use crate::style;
use crate::GitCommands;
use pctrl_core::{Config, GitRepo};
use pctrl_database::Database;
use pctrl_git::GitManager;

pub async fn handle(command: GitCommands, config: &Config, db: &Database) -> anyhow::Result<()> {
    style::deprecation_warning("git", "project");

    // Initialize manager with config data
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
                anyhow::bail!(
                    "Git repository '{}' already exists. Use a different name.",
                    id
                );
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
