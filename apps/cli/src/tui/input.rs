//! TUI input handling

use super::app::App;
use super::types::{InputMode, SelectedPanel};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use pctrl_core::{
    DatabaseCredentials, DatabaseType, Domain, DomainType, Project, ProjectStatus, Script,
    ScriptType, Server, ServerType,
};
use std::io;
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
                        SelectedPanel::Projects => SelectedPanel::Servers,
                        SelectedPanel::Servers => SelectedPanel::Domains,
                        SelectedPanel::Domains => SelectedPanel::Databases,
                        SelectedPanel::Databases => SelectedPanel::Scripts,
                        SelectedPanel::Scripts => SelectedPanel::Status,
                    };
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.selected_panel = match app.selected_panel {
                        SelectedPanel::Status => SelectedPanel::Scripts,
                        SelectedPanel::Projects => SelectedPanel::Status,
                        SelectedPanel::Servers => SelectedPanel::Projects,
                        SelectedPanel::Domains => SelectedPanel::Servers,
                        SelectedPanel::Databases => SelectedPanel::Domains,
                        SelectedPanel::Scripts => SelectedPanel::Databases,
                    };
                }
                KeyCode::Char('a') => {
                    if app.selected_panel != SelectedPanel::Status {
                        app.reset_form();
                        app.input_mode = InputMode::Adding;
                    }
                }
                KeyCode::Char('r') => {
                    app.load_all().await;
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
                        app.load_all().await;
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

    match app.selected_panel {
        SelectedPanel::Projects => {
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
        }
        SelectedPanel::Servers => {
            if app.input_form.name.is_empty() || app.input_form.host.is_empty() {
                anyhow::bail!("Name and Host are required");
            }

            let server_type: ServerType = app
                .input_form
                .server_type
                .parse()
                .unwrap_or(ServerType::Vps);

            let server = Server {
                id,
                name: app.input_form.name.clone(),
                host: app.input_form.host.clone(),
                server_type,
                provider: if app.input_form.provider.is_empty() {
                    None
                } else {
                    Some(app.input_form.provider.clone())
                },
                credential_id: None,
                location: None,
                specs: None,
                notes: None,
            };

            app.db.save_server(&server).await?;
        }
        SelectedPanel::Domains => {
            if app.input_form.domain.is_empty() {
                anyhow::bail!("Domain is required");
            }

            let domain_type: DomainType = app
                .input_form
                .domain_type
                .parse()
                .unwrap_or(DomainType::Production);

            let ssl = app.input_form.ssl.to_lowercase() == "true"
                || app.input_form.ssl.to_lowercase() == "yes"
                || app.input_form.ssl == "1";

            let domain = Domain {
                id,
                domain: app.input_form.domain.clone(),
                domain_type,
                ssl,
                ssl_expiry: None,
                cloudflare_zone_id: None,
                cloudflare_record_id: None,
                server_id: None,
                container_id: None,
                notes: None,
            };

            app.db.save_domain(&domain).await?;
        }
        SelectedPanel::Databases => {
            if app.input_form.name.is_empty() {
                anyhow::bail!("Name is required");
            }

            let port: Option<u16> = if app.input_form.port.is_empty() {
                None
            } else {
                app.input_form.port.parse().ok()
            };

            let db_type: DatabaseType = app
                .input_form
                .db_type
                .parse()
                .unwrap_or(DatabaseType::PostgreSQL);

            let host = if app.input_form.host.is_empty() {
                None
            } else {
                Some(app.input_form.host.clone())
            };

            let database = DatabaseCredentials {
                id,
                name: app.input_form.name.clone(),
                db_type,
                host,
                port,
                database_name: None,
                username: if app.input_form.user.is_empty() {
                    None
                } else {
                    Some(app.input_form.user.clone())
                },
                password: if app.input_form.password.is_empty() {
                    None
                } else {
                    Some(app.input_form.password.clone())
                },
                connection_string: None,
                server_id: None,
                container_id: None,
                notes: None,
            };

            app.db.save_database_credentials(&database).await?;
        }
        SelectedPanel::Scripts => {
            if app.input_form.name.is_empty() || app.input_form.command.is_empty() {
                anyhow::bail!("Name and Command are required");
            }

            let script_type: ScriptType = app
                .input_form
                .script_type
                .parse()
                .unwrap_or(ScriptType::Local);

            let script = Script {
                id,
                name: app.input_form.name.clone(),
                description: None,
                command: app.input_form.command.clone(),
                script_type,
                server_id: None,
                project_id: None,
                docker_host_id: None,
                container_id: None,
                dangerous: false,
                last_run: None,
                last_result: None,
                exit_code: None,
                last_output: None,
            };

            app.db.save_script(&script).await?;
        }
        SelectedPanel::Status => {}
    }

    Ok(())
}
