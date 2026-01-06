use serde::{Deserialize, Serialize};
use std::fmt;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_path: String,
    pub ssh_connections: Vec<SshConnection>,
    pub docker_hosts: Vec<DockerHost>,
    pub coolify_instances: Vec<CoolifyInstance>,
    pub git_repos: Vec<GitRepo>,
}

/// SSH connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Password,
    PublicKey { key_path: String },
}

/// Docker host configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerHost {
    pub id: String,
    pub name: String,
    pub url: String,
}

/// Coolify instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoolifyInstance {
    pub id: String,
    pub name: String,
    pub url: String,
    pub api_key: String,
}

/// Git repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub remote_url: Option<String>,
}

/// Application error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("SSH error: {0}")]
    Ssh(String),
    
    #[error("Docker error: {0}")]
    Docker(String),
    
    #[error("Coolify error: {0}")]
    Coolify(String),
    
    #[error("Git error: {0}")]
    Git(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Mode of operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Cli,
    Tui,
    Gui,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Cli => write!(f, "cli"),
            Mode::Tui => write!(f, "tui"),
            Mode::Gui => write!(f, "gui"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_path: "pctrl.db".to_string(),
            ssh_connections: Vec::new(),
            docker_hosts: Vec::new(),
            coolify_instances: Vec::new(),
            git_repos: Vec::new(),
        }
    }
}
