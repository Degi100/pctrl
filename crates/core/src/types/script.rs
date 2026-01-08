//! Script types

use serde::{Deserialize, Serialize};
use std::fmt;

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
    /// Docker host ID for docker scripts
    pub docker_host_id: Option<String>,
    /// Container ID/name for docker scripts
    pub container_id: Option<String>,
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
