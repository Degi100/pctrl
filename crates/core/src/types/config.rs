//! Application configuration

use super::{
    CoolifyInstance, DatabaseCredentials, DockerHost, Domain, GitRepo, Project, Script, Server,
    SshConnection,
};
use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_path: String,
    pub ssh_connections: Vec<SshConnection>,
    pub docker_hosts: Vec<DockerHost>,
    pub coolify_instances: Vec<CoolifyInstance>,
    pub git_repos: Vec<GitRepo>,
    // v6: New entities
    pub projects: Vec<Project>,
    pub servers: Vec<Server>,
    pub domains: Vec<Domain>,
    pub databases: Vec<DatabaseCredentials>,
    pub scripts: Vec<Script>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_path: "pctrl.db".to_string(),
            ssh_connections: Vec::new(),
            docker_hosts: Vec::new(),
            coolify_instances: Vec::new(),
            git_repos: Vec::new(),
            projects: Vec::new(),
            servers: Vec::new(),
            domains: Vec::new(),
            databases: Vec::new(),
            scripts: Vec::new(),
        }
    }
}

/// Mode of operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Cli,
    Tui,
    Gui,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Cli => write!(f, "cli"),
            Mode::Tui => write!(f, "tui"),
            Mode::Gui => write!(f, "gui"),
        }
    }
}
