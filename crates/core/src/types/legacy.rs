//! Legacy types for backwards compatibility

use serde::{Deserialize, Serialize};

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
