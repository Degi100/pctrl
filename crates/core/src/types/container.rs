//! Container types

use serde::{Deserialize, Serialize};
use std::fmt;

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
