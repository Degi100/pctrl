use serde::{Deserialize, Serialize};
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════════
// APPLICATION CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════════════
// PROJECT - Core Entity (v6)
// ═══════════════════════════════════════════════════════════════════════════════

/// Project - Das Herzstück von pctrl v6
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub stack: Vec<String>,
    pub status: ProjectStatus,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ProjectStatus {
    #[default]
    Dev,
    Staging,
    Live,
    Archived,
}

impl fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectStatus::Dev => write!(f, "dev"),
            ProjectStatus::Staging => write!(f, "staging"),
            ProjectStatus::Live => write!(f, "live"),
            ProjectStatus::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for ProjectStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dev" => Ok(ProjectStatus::Dev),
            "staging" => Ok(ProjectStatus::Staging),
            "live" => Ok(ProjectStatus::Live),
            "archived" => Ok(ProjectStatus::Archived),
            _ => Err(format!("Unknown project status: {}", s)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SERVER - Eigenständige Entity (v6)
// ═══════════════════════════════════════════════════════════════════════════════

/// Server - eigenständige Entity (nicht nur SSH)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub host: String,
    pub server_type: ServerType,
    pub provider: Option<String>,
    pub ssh_connection_id: Option<String>,
    pub location: Option<String>,
    pub specs: Option<ServerSpecs>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSpecs {
    pub cpu_cores: Option<u8>,
    pub ram_gb: Option<u16>,
    pub disk_gb: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ServerType {
    #[default]
    Vps,
    Dedicated,
    Local,
    Cloud,
}

impl fmt::Display for ServerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerType::Vps => write!(f, "vps"),
            ServerType::Dedicated => write!(f, "dedicated"),
            ServerType::Local => write!(f, "local"),
            ServerType::Cloud => write!(f, "cloud"),
        }
    }
}

impl std::str::FromStr for ServerType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vps" => Ok(ServerType::Vps),
            "dedicated" => Ok(ServerType::Dedicated),
            "local" => Ok(ServerType::Local),
            "cloud" => Ok(ServerType::Cloud),
            _ => Err(format!("Unknown server type: {}", s)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DOMAIN (v6)
// ═══════════════════════════════════════════════════════════════════════════════

/// Domain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: String,
    pub domain: String,
    pub domain_type: DomainType,
    pub ssl: bool,
    pub ssl_expiry: Option<String>,
    pub cloudflare_zone_id: Option<String>,
    pub cloudflare_record_id: Option<String>,
    pub server_id: Option<String>,
    pub container_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DomainType {
    #[default]
    Production,
    Staging,
    Dev,
}

impl fmt::Display for DomainType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainType::Production => write!(f, "production"),
            DomainType::Staging => write!(f, "staging"),
            DomainType::Dev => write!(f, "dev"),
        }
    }
}

impl std::str::FromStr for DomainType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Ok(DomainType::Production),
            "staging" => Ok(DomainType::Staging),
            "dev" => Ok(DomainType::Dev),
            _ => Err(format!("Unknown domain type: {}", s)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DATABASE CREDENTIALS (v6) - Encrypted!
// ═══════════════════════════════════════════════════════════════════════════════

/// Database Credentials (encrypted storage)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseCredentials {
    pub id: String,
    pub name: String,
    pub db_type: DatabaseType,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database_name: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub connection_string: Option<String>,
    pub server_id: Option<String>,
    pub container_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DatabaseType {
    MongoDB,
    #[default]
    PostgreSQL,
    MySQL,
    Redis,
    SQLite,
}

impl fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseType::MongoDB => write!(f, "mongodb"),
            DatabaseType::PostgreSQL => write!(f, "postgres"),
            DatabaseType::MySQL => write!(f, "mysql"),
            DatabaseType::Redis => write!(f, "redis"),
            DatabaseType::SQLite => write!(f, "sqlite"),
        }
    }
}

impl std::str::FromStr for DatabaseType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mongodb" | "mongo" => Ok(DatabaseType::MongoDB),
            "postgresql" | "postgres" | "pg" => Ok(DatabaseType::PostgreSQL),
            "mysql" | "mariadb" => Ok(DatabaseType::MySQL),
            "redis" => Ok(DatabaseType::Redis),
            "sqlite" => Ok(DatabaseType::SQLite),
            _ => Err(format!("Unknown database type: {}", s)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTAINER (v6) - Erweitert
// ═══════════════════════════════════════════════════════════════════════════════

/// Container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: Option<String>,
    pub server_id: String,
    pub project_id: Option<String>,
    pub status: ContainerStatus,
    pub ports: Vec<String>,
    pub env_vars: Option<String>,
    pub labels: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ContainerStatus {
    Running,
    Stopped,
    Restarting,
    Paused,
    Exited,
    #[default]
    Unknown,
}

impl fmt::Display for ContainerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerStatus::Running => write!(f, "running"),
            ContainerStatus::Stopped => write!(f, "stopped"),
            ContainerStatus::Restarting => write!(f, "restarting"),
            ContainerStatus::Paused => write!(f, "paused"),
            ContainerStatus::Exited => write!(f, "exited"),
            ContainerStatus::Unknown => write!(f, "unknown"),
        }
    }
}

impl std::str::FromStr for ContainerStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "running" => Ok(ContainerStatus::Running),
            "stopped" => Ok(ContainerStatus::Stopped),
            "restarting" => Ok(ContainerStatus::Restarting),
            "paused" => Ok(ContainerStatus::Paused),
            "exited" => Ok(ContainerStatus::Exited),
            _ => Ok(ContainerStatus::Unknown),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCRIPT (v6)
// ═══════════════════════════════════════════════════════════════════════════════

/// Script for automation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub command: String,
    pub script_type: ScriptType,
    pub server_id: Option<String>,
    pub project_id: Option<String>,
    pub dangerous: bool,
    pub last_run: Option<String>,
    pub last_result: Option<ScriptResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ScriptType {
    #[default]
    Ssh,
    Local,
    Docker,
}

impl fmt::Display for ScriptType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptType::Ssh => write!(f, "ssh"),
            ScriptType::Local => write!(f, "local"),
            ScriptType::Docker => write!(f, "docker"),
        }
    }
}

impl std::str::FromStr for ScriptType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ssh" => Ok(ScriptType::Ssh),
            "local" => Ok(ScriptType::Local),
            "docker" => Ok(ScriptType::Docker),
            _ => Err(format!("Unknown script type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScriptResult {
    Success,
    Error,
}

impl fmt::Display for ScriptResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptResult::Success => write!(f, "success"),
            ScriptResult::Error => write!(f, "error"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PROJECT RESOURCES (v6) - Verknüpfungstabelle
// ═══════════════════════════════════════════════════════════════════════════════

/// Project resource link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResource {
    pub id: String,
    pub project_id: String,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub role: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceType {
    Server,
    Container,
    Database,
    Domain,
    Git,
    Coolify,
    Script,
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Server => write!(f, "server"),
            ResourceType::Container => write!(f, "container"),
            ResourceType::Database => write!(f, "database"),
            ResourceType::Domain => write!(f, "domain"),
            ResourceType::Git => write!(f, "git"),
            ResourceType::Coolify => write!(f, "coolify"),
            ResourceType::Script => write!(f, "script"),
        }
    }
}

impl std::str::FromStr for ResourceType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "server" => Ok(ResourceType::Server),
            "container" => Ok(ResourceType::Container),
            "database" => Ok(ResourceType::Database),
            "domain" => Ok(ResourceType::Domain),
            "git" => Ok(ResourceType::Git),
            "coolify" => Ok(ResourceType::Coolify),
            "script" => Ok(ResourceType::Script),
            _ => Err(format!("Unknown resource type: {}", s)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LEGACY TYPES (behalten für Kompatibilität)
// ═══════════════════════════════════════════════════════════════════════════════

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
            // v6: New entities
            projects: Vec::new(),
            servers: Vec::new(),
            domains: Vec::new(),
            databases: Vec::new(),
            scripts: Vec::new(),
        }
    }
}
