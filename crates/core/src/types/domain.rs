//! Domain types

use serde::{Deserialize, Serialize};
use std::fmt;

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
