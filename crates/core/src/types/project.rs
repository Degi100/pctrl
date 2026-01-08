//! Project types

use serde::{Deserialize, Serialize};
use std::fmt;

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
