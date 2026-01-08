//! Docker Host CRUD operations

use crate::Database;
use pctrl_core::Result;
use sqlx::Row;

impl Database {
    /// Add or update a Docker host
    pub async fn save_docker_host(&self, host: &pctrl_core::DockerHost) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO docker_hosts (id, name, url)
             VALUES (?, ?, ?)",
        )
        .bind(&host.id)
        .bind(&host.name)
        .bind(&host.url)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Remove a Docker host by ID
    pub async fn remove_docker_host(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM docker_hosts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if a Docker host exists
    pub async fn docker_host_exists(&self, id: &str) -> Result<bool> {
        let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM docker_hosts WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(|(count,)| count > 0).unwrap_or(false))
    }

    /// Load all Docker hosts
    pub(crate) async fn load_docker_hosts(&self) -> Result<Vec<pctrl_core::DockerHost>> {
        let rows = sqlx::query("SELECT id, name, url FROM docker_hosts")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let hosts = rows
            .into_iter()
            .map(|row| pctrl_core::DockerHost {
                id: row.get("id"),
                name: row.get("name"),
                url: row.get("url"),
            })
            .collect();

        Ok(hosts)
    }
}
