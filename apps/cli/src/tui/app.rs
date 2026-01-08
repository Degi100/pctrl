//! TUI application state

use super::checks::{check_coolify_connections, check_docker_connections, check_ssh_connections};
use super::types::{ConnectionStatus, InputForm, InputMode, SelectedPanel};
use pctrl_core::{Config, Project};
use pctrl_database::Database;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub struct App {
    pub selected_panel: SelectedPanel,
    pub config: Arc<Config>,
    pub db: Arc<Database>,
    pub projects: Vec<Project>,
    pub ssh_status: HashMap<String, ConnectionStatus>,
    pub docker_status: HashMap<String, ConnectionStatus>,
    pub coolify_status: HashMap<String, ConnectionStatus>,
    pub git_status: HashMap<String, ConnectionStatus>,
    pub input_mode: InputMode,
    pub input_form: InputForm,
    pub checking_connections: bool,
}

impl App {
    pub fn new(config: Arc<Config>, db: Arc<Database>) -> Self {
        let ssh_status: HashMap<String, ConnectionStatus> = config
            .ssh_connections
            .iter()
            .map(|c| (c.id.clone(), ConnectionStatus::Unknown))
            .collect();
        let docker_status: HashMap<String, ConnectionStatus> = config
            .docker_hosts
            .iter()
            .map(|h| (h.id.clone(), ConnectionStatus::Unknown))
            .collect();
        let coolify_status: HashMap<String, ConnectionStatus> = config
            .coolify_instances
            .iter()
            .map(|i| (i.id.clone(), ConnectionStatus::Unknown))
            .collect();
        let git_status: HashMap<String, ConnectionStatus> = config
            .git_repos
            .iter()
            .map(|r| (r.id.clone(), ConnectionStatus::Unknown))
            .collect();

        Self {
            selected_panel: SelectedPanel::Status,
            config,
            db,
            projects: Vec::new(),
            ssh_status,
            docker_status,
            coolify_status,
            git_status,
            input_mode: InputMode::Normal,
            input_form: InputForm::default(),
            checking_connections: false,
        }
    }

    pub async fn load_projects(&mut self) {
        if let Ok(projects) = self.db.list_projects().await {
            self.projects = projects;
        }
    }

    pub fn get_form_fields(&self) -> Vec<(&'static str, &str)> {
        match self.selected_panel {
            SelectedPanel::Projects => vec![
                ("Name", &self.input_form.name),
                ("Description", &self.input_form.description),
                ("Stack", &self.input_form.stack),
                ("Status", &self.input_form.status),
            ],
            SelectedPanel::Ssh => vec![
                ("Name", &self.input_form.name),
                ("Host", &self.input_form.host),
                ("User", &self.input_form.user),
                ("Port", &self.input_form.port),
            ],
            SelectedPanel::Docker => vec![
                ("Name", &self.input_form.name),
                ("URL", &self.input_form.url),
            ],
            SelectedPanel::Coolify => vec![
                ("Name", &self.input_form.name),
                ("URL", &self.input_form.url),
                ("Token", &self.input_form.token),
            ],
            SelectedPanel::Git => vec![
                ("Name", &self.input_form.name),
                ("Path", &self.input_form.path),
            ],
            SelectedPanel::Status => vec![],
        }
    }

    pub fn current_input_mut(&mut self) -> Option<&mut String> {
        let field_idx = self.input_form.current_field;
        match self.selected_panel {
            SelectedPanel::Projects => match field_idx {
                0 => Some(&mut self.input_form.name),
                1 => Some(&mut self.input_form.description),
                2 => Some(&mut self.input_form.stack),
                3 => Some(&mut self.input_form.status),
                _ => None,
            },
            SelectedPanel::Ssh => match field_idx {
                0 => Some(&mut self.input_form.name),
                1 => Some(&mut self.input_form.host),
                2 => Some(&mut self.input_form.user),
                3 => Some(&mut self.input_form.port),
                _ => None,
            },
            SelectedPanel::Docker => match field_idx {
                0 => Some(&mut self.input_form.name),
                1 => Some(&mut self.input_form.url),
                _ => None,
            },
            SelectedPanel::Coolify => match field_idx {
                0 => Some(&mut self.input_form.name),
                1 => Some(&mut self.input_form.url),
                2 => Some(&mut self.input_form.token),
                _ => None,
            },
            SelectedPanel::Git => match field_idx {
                0 => Some(&mut self.input_form.name),
                1 => Some(&mut self.input_form.path),
                _ => None,
            },
            SelectedPanel::Status => None,
        }
    }

    pub fn field_count(&self) -> usize {
        match self.selected_panel {
            SelectedPanel::Projects => 4,
            SelectedPanel::Ssh => 4,
            SelectedPanel::Docker => 2,
            SelectedPanel::Coolify => 3,
            SelectedPanel::Git => 2,
            SelectedPanel::Status => 0,
        }
    }

    pub fn reset_form(&mut self) {
        self.input_form = InputForm::default();
        match self.selected_panel {
            SelectedPanel::Projects => {
                self.input_form.status = "dev".to_string();
            }
            SelectedPanel::Ssh => {
                self.input_form.port = "22".to_string();
            }
            SelectedPanel::Docker => {
                self.input_form.url = "unix:///var/run/docker.sock".to_string();
            }
            _ => {}
        }
    }

    pub async fn check_all_connections(&mut self) {
        self.checking_connections = true;

        // Git repos (path check)
        for repo in &self.config.git_repos {
            let status = if Path::new(&repo.path).exists() {
                ConnectionStatus::Online
            } else {
                ConnectionStatus::Offline
            };
            self.git_status.insert(repo.id.clone(), status);
        }

        // SSH connections
        let ssh_connections = self.config.ssh_connections.clone();
        let ssh_results = check_ssh_connections(ssh_connections).await;
        for (id, status) in ssh_results {
            self.ssh_status.insert(id, status);
        }

        // Docker hosts
        let docker_hosts = self.config.docker_hosts.clone();
        let docker_results = check_docker_connections(docker_hosts).await;
        for (id, status) in docker_results {
            self.docker_status.insert(id, status);
        }

        // Coolify instances
        let coolify_instances = self.config.coolify_instances.clone();
        let coolify_results = check_coolify_connections(coolify_instances).await;
        for (id, status) in coolify_results {
            self.coolify_status.insert(id, status);
        }

        self.checking_connections = false;
    }

    pub fn count_by_status(
        &self,
        statuses: &HashMap<String, ConnectionStatus>,
    ) -> (usize, usize, usize) {
        let online = statuses
            .values()
            .filter(|s| **s == ConnectionStatus::Online)
            .count();
        let offline = statuses
            .values()
            .filter(|s| **s == ConnectionStatus::Offline)
            .count();
        let unknown = statuses
            .values()
            .filter(|s| **s == ConnectionStatus::Unknown)
            .count();
        (online, offline, unknown)
    }
}
