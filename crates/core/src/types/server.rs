//! Server types

use serde::{Deserialize, Serialize};
use std::fmt;

/// Server - eigenständige Entity (nicht nur SSH)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub host: String,
    pub server_type: ServerType,
    pub provider: Option<String>,
    /// Reference to a Credential for SSH access
    pub credential_id: Option<String>,
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
