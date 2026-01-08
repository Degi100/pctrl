//! Core types for pctrl
//!
//! This module contains all the data structures used throughout pctrl.

mod config;
mod container;
mod database;
mod domain;
mod error;
mod legacy;
mod project;
mod resource;
mod script;
mod server;

// Re-export all types
pub use config::{Config, Mode};
pub use container::{Container, ContainerStatus};
pub use database::{DatabaseCredentials, DatabaseType};
pub use domain::{Domain, DomainType};
pub use error::{Error, Result};
pub use legacy::{AuthMethod, CoolifyInstance, DockerHost, GitRepo, SshConnection};
pub use project::{Project, ProjectStatus};
pub use resource::{ProjectResource, ResourceType};
pub use script::{Script, ScriptResult, ScriptType};
pub use server::{Server, ServerSpecs, ServerType};
