//! Config save/load operations

use crate::Database;
use pctrl_core::{Config, Result};

impl Database {
    /// Save configuration to database
    pub async fn save_config(&self, config: &Config) -> Result<()> {
        // Save SSH connections
        for conn in &config.ssh_connections {
            self.save_ssh_connection(conn).await?;
        }

        // Note: Docker hosts, Coolify instances, and Git repos are typically
        // saved individually through their respective methods.
        // This method primarily ensures SSH connections are persisted.

        Ok(())
    }

    /// Load configuration from database
    #[allow(clippy::field_reassign_with_default)]
    pub async fn load_config(&self) -> Result<Config> {
        let mut config = Config::default();

        // Load all entity types
        config.ssh_connections = self.load_ssh_connections().await?;
        config.docker_hosts = self.load_docker_hosts().await?;
        config.coolify_instances = self.load_coolify_instances().await?;
        config.git_repos = self.load_git_repos().await?;

        Ok(config)
    }
}
