//! Project resource linking types

use serde::{Deserialize, Serialize};
use std::fmt;

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
