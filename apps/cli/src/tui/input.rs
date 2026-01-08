//! TUI input handling

use super::app::App;
use super::types::{ConnectionStatus, InputMode, SelectedPanel};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use pctrl_core::{
    AuthMethod, CoolifyInstance, DockerHost, GitRepo, Project, ProjectStatus, SshConnection,
};
use std::io;
use std::sync::Arc;
use uuid::Uuid;

/// Handle keyboard input, returns true if should quit
pub async fn handle_input(app: &mut App, event: Event) -> io::Result<bool> {
    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return Ok(false);
        }

        match app.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                KeyCode::Down | KeyCode::Char('j') => {
                    app.selected_panel = match app.selected_panel {
                        SelectedPanel::Status => SelectedPanel::Projects,
                        SelectedPanel::Projects => SelectedPanel::Ssh,
                        SelectedPanel::Ssh => SelectedPanel::Docker,
                        SelectedPanel::Docker => SelectedPanel::Coolify,
                        SelectedPanel::Coolify => SelectedPanel::Git,
                        SelectedPanel::Git => SelectedPanel::Status,
                    };
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.selected_panel = match app.selected_panel {
                        SelectedPanel::Status => SelectedPanel::Git,
                        SelectedPanel::Projects => SelectedPanel::Status,
                        SelectedPanel::Ssh => SelectedPanel::Projects,
                        SelectedPanel::Docker => SelectedPanel::Ssh,
                        SelectedPanel::Coolify => SelectedPanel::Docker,
                        SelectedPanel::Git => SelectedPanel::Coolify,
                    };
                }
                KeyCode::Char('a') => {
                    if app.selected_panel != SelectedPanel::Status {
                        app.reset_form();
                        app.input_mode = InputMode::Adding;
                    }
                }
                KeyCode::Char('r') => {
                    app.check_all_connections().await;
                }
                _ => {}
            },
            InputMode::Adding => match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    app.reset_form();
                }
                KeyCode::Tab => {
                    let count = app.field_count();
                    if count > 0 {
                        app.input_form.current_field = (app.input_form.current_field + 1) % count;
                    }
                }
                KeyCode::BackTab => {
                    let count = app.field_count();
                    if count > 0 {
                        app.input_form.current_field = if app.input_form.current_field == 0 {
                            count - 1
                        } else {
                            app.input_form.current_field - 1
                        };
                    }
                }
                KeyCode::Enter => {
                    if let Err(e) = save_new_entry(app).await {
                        app.input_form.message = Some(format!("Error: {}", e));
                    } else {
                        app.input_mode = InputMode::Normal;
                        app.reset_form();
                        app.check_all_connections().await;
                    }
                }
                KeyCode::Backspace => {
                    if let Some(input) = app.current_input_mut() {
                        input.pop();
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(input) = app.current_input_mut() {
                        input.push(c);
                    }
                }
                _ => {}
            },
        }
    }
    Ok(false)
}

/// Save a new entry based on current panel and form data
async fn save_new_entry(app: &mut App) -> anyhow::Result<()> {
    let id = Uuid::new_v4().to_string();

    // Handle Projects separately (saved to database)
    if app.selected_panel == SelectedPanel::Projects {
        if app.input_form.name.is_empty() {
            anyhow::bail!("Name is required");
        }

        let stack: Vec<String> = if app.input_form.stack.is_empty() {
            vec![]
        } else {
            app.input_form
                .stack
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        let status: ProjectStatus = app.input_form.status.parse().unwrap_or_default();

        let project = Project {
            id,
            name: app.input_form.name.clone(),
            description: if app.input_form.description.is_empty() {
                None
            } else {
                Some(app.input_form.description.clone())
            },
            stack,
            status,
            color: None,
            icon: None,
            notes: None,
        };

        app.db.save_project(&project).await?;
        app.projects.push(project);
        return Ok(());
    }

    // Get mutable config
    let config = Arc::make_mut(&mut app.config);

    match app.selected_panel {
        SelectedPanel::Projects => unreachable!(),
        SelectedPanel::Ssh => {
            if app.input_form.name.is_empty() || app.input_form.host.is_empty() {
                anyhow::bail!("Name and Host are required");
            }
            let port: u16 = app.input_form.port.parse().unwrap_or(22);
            let username = if app.input_form.user.is_empty() {
                "root".to_string()
            } else {
                app.input_form.user.clone()
            };

            let conn = SshConnection {
                id: id.clone(),
                name: app.input_form.name.clone(),
                host: app.input_form.host.clone(),
                port,
                username,
                auth_method: AuthMethod::PublicKey {
                    key_path: "~/.ssh/id_rsa".to_string(),
                },
            };
            config.ssh_connections.push(conn);
            app.ssh_status.insert(id, ConnectionStatus::Unknown);
        }
        SelectedPanel::Docker => {
            if app.input_form.name.is_empty() {
                anyhow::bail!("Name is required");
            }
            let url = if app.input_form.url.is_empty() {
                "unix:///var/run/docker.sock".to_string()
            } else {
                app.input_form.url.clone()
            };

            let host = DockerHost {
                id: id.clone(),
                name: app.input_form.name.clone(),
                url,
            };
            config.docker_hosts.push(host);
            app.docker_status.insert(id, ConnectionStatus::Unknown);
        }
        SelectedPanel::Coolify => {
            if app.input_form.name.is_empty()
                || app.input_form.url.is_empty()
                || app.input_form.token.is_empty()
            {
                anyhow::bail!("Name, URL, and Token are required");
            }

            let instance = CoolifyInstance {
                id: id.clone(),
                name: app.input_form.name.clone(),
                url: app.input_form.url.clone(),
                api_key: app.input_form.token.clone(),
            };
            config.coolify_instances.push(instance);
            app.coolify_status.insert(id, ConnectionStatus::Unknown);
        }
        SelectedPanel::Git => {
            if app.input_form.name.is_empty() || app.input_form.path.is_empty() {
                anyhow::bail!("Name and Path are required");
            }

            let repo = GitRepo {
                id: id.clone(),
                name: app.input_form.name.clone(),
                path: app.input_form.path.clone(),
                remote_url: None,
            };
            config.git_repos.push(repo);
            app.git_status.insert(id, ConnectionStatus::Unknown);
        }
        SelectedPanel::Status => {}
    }

    // Save config to database
    app.db.save_config(config).await?;

    Ok(())
}
