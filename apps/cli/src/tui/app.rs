//! TUI application state

use super::types::{InputForm, InputMode, SelectedPanel};
use pctrl_core::{DatabaseCredentials, Domain, Project, Script, Server};
use pctrl_database::Database;
use std::sync::Arc;

pub struct App {
    pub selected_panel: SelectedPanel,
    pub db: Arc<Database>,
    // v6 entities
    pub projects: Vec<Project>,
    pub servers: Vec<Server>,
    pub domains: Vec<Domain>,
    pub databases: Vec<DatabaseCredentials>,
    pub scripts: Vec<Script>,
    // Legacy counts (for migration warning)
    pub legacy_ssh_count: usize,
    pub legacy_docker_count: usize,
    pub legacy_coolify_count: usize,
    pub legacy_git_count: usize,
    // UI state
    pub input_mode: InputMode,
    pub input_form: InputForm,
    pub loading: bool,
}

impl App {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            selected_panel: SelectedPanel::Status,
            db,
            projects: Vec::new(),
            servers: Vec::new(),
            domains: Vec::new(),
            databases: Vec::new(),
            scripts: Vec::new(),
            legacy_ssh_count: 0,
            legacy_docker_count: 0,
            legacy_coolify_count: 0,
            legacy_git_count: 0,
            input_mode: InputMode::Normal,
            input_form: InputForm::default(),
            loading: false,
        }
    }

    pub async fn load_all(&mut self) {
        self.loading = true;

        // Load v6 entities
        if let Ok(projects) = self.db.list_projects().await {
            self.projects = projects;
        }
        if let Ok(servers) = self.db.list_servers().await {
            self.servers = servers;
        }
        if let Ok(domains) = self.db.list_domains().await {
            self.domains = domains;
        }
        if let Ok(databases) = self.db.list_database_credentials().await {
            self.databases = databases;
        }
        if let Ok(scripts) = self.db.list_scripts().await {
            self.scripts = scripts;
        }

        // Load legacy counts for migration warning
        if let Ok(config) = self.db.load_config().await {
            self.legacy_ssh_count = config.ssh_connections.len();
            self.legacy_docker_count = config.docker_hosts.len();
            self.legacy_coolify_count = config.coolify_instances.len();
            self.legacy_git_count = config.git_repos.len();
        }

        self.loading = false;
    }

    pub fn total_legacy_count(&self) -> usize {
        self.legacy_ssh_count
            + self.legacy_docker_count
            + self.legacy_coolify_count
            + self.legacy_git_count
    }

    pub fn get_form_fields(&self) -> Vec<(&'static str, &str)> {
        match self.selected_panel {
            SelectedPanel::Projects => vec![
                ("Name", &self.input_form.name),
                ("Description", &self.input_form.description),
                ("Stack", &self.input_form.stack),
                ("Status", &self.input_form.status),
            ],
            SelectedPanel::Servers => vec![
                ("Name", &self.input_form.name),
                ("Host", &self.input_form.host),
                ("Type", &self.input_form.server_type),
                ("Provider", &self.input_form.provider),
            ],
            SelectedPanel::Domains => vec![
                ("Domain", &self.input_form.domain),
                ("Type", &self.input_form.domain_type),
                ("SSL", &self.input_form.ssl),
            ],
            SelectedPanel::Databases => vec![
                ("Name", &self.input_form.name),
                ("Type", &self.input_form.db_type),
                ("Host", &self.input_form.host),
                ("Port", &self.input_form.port),
            ],
            SelectedPanel::Scripts => vec![
                ("Name", &self.input_form.name),
                ("Command", &self.input_form.command),
                ("Type", &self.input_form.script_type),
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
            SelectedPanel::Servers => match field_idx {
                0 => Some(&mut self.input_form.name),
                1 => Some(&mut self.input_form.host),
                2 => Some(&mut self.input_form.server_type),
                3 => Some(&mut self.input_form.provider),
                _ => None,
            },
            SelectedPanel::Domains => match field_idx {
                0 => Some(&mut self.input_form.domain),
                1 => Some(&mut self.input_form.domain_type),
                2 => Some(&mut self.input_form.ssl),
                _ => None,
            },
            SelectedPanel::Databases => match field_idx {
                0 => Some(&mut self.input_form.name),
                1 => Some(&mut self.input_form.db_type),
                2 => Some(&mut self.input_form.host),
                3 => Some(&mut self.input_form.port),
                _ => None,
            },
            SelectedPanel::Scripts => match field_idx {
                0 => Some(&mut self.input_form.name),
                1 => Some(&mut self.input_form.command),
                2 => Some(&mut self.input_form.script_type),
                _ => None,
            },
            SelectedPanel::Status => None,
        }
    }

    pub fn field_count(&self) -> usize {
        match self.selected_panel {
            SelectedPanel::Projects => 4,
            SelectedPanel::Servers => 4,
            SelectedPanel::Domains => 3,
            SelectedPanel::Databases => 4,
            SelectedPanel::Scripts => 3,
            SelectedPanel::Status => 0,
        }
    }

    pub fn reset_form(&mut self) {
        self.input_form = InputForm::default();
        match self.selected_panel {
            SelectedPanel::Projects => {
                self.input_form.status = "dev".to_string();
            }
            SelectedPanel::Servers => {
                self.input_form.server_type = "vps".to_string();
            }
            SelectedPanel::Domains => {
                self.input_form.domain_type = "production".to_string();
                self.input_form.ssl = "true".to_string();
            }
            SelectedPanel::Databases => {
                self.input_form.db_type = "postgres".to_string();
                self.input_form.port = "5432".to_string();
            }
            SelectedPanel::Scripts => {
                self.input_form.script_type = "local".to_string();
            }
            _ => {}
        }
    }
}
