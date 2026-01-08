//! Database credential types

use serde::{Deserialize, Serialize};
use std::fmt;

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
