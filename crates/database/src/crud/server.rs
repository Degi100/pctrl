//! Server CRUD operations

use crate::Database;
use pctrl_core::Result;

impl Database {
    /// Save a server
    pub async fn save_server(&self, server: &pctrl_core::Server) -> Result<()> {
        let specs = server
            .specs
            .as_ref()
            .map(|s| serde_json::to_string(s).unwrap_or_default());

        sqlx::query(
            "INSERT OR REPLACE INTO servers (id, name, host, server_type, provider, ssh_connection_id, location, specs, notes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&server.id)
        .bind(&server.name)
        .bind(&server.host)
        .bind(server.server_type.to_string())
        .bind(&server.provider)
        .bind(&server.ssh_connection_id)
        .bind(&server.location)
        .bind(&specs)
        .bind(&server.notes)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Get a server by ID
    pub async fn get_server(&self, id: &str) -> Result<Option<pctrl_core::Server>> {
        let row: Option<(
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, host, server_type, provider, ssh_connection_id, location, specs, notes FROM servers WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(Self::row_to_server))
    }

    /// Get a server by name (case-insensitive)
    pub async fn get_server_by_name(&self, name: &str) -> Result<Option<pctrl_core::Server>> {
        let row: Option<(
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, host, server_type, provider, ssh_connection_id, location, specs, notes FROM servers WHERE LOWER(name) = LOWER(?)",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(Self::row_to_server))
    }

    /// List all servers
    pub async fn list_servers(&self) -> Result<Vec<pctrl_core::Server>> {
        let rows: Vec<(
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, host, server_type, provider, ssh_connection_id, location, specs, notes FROM servers ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(rows.into_iter().map(Self::row_to_server).collect())
    }

    /// Remove a server by ID
    pub async fn remove_server(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM servers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if a server exists
    pub async fn server_exists(&self, id: &str) -> Result<bool> {
        let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM servers WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(|(count,)| count > 0).unwrap_or(false))
    }

    /// Helper to convert a row tuple to Server
    fn row_to_server(
        row: (
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    ) -> pctrl_core::Server {
        let (id, name, host, server_type, provider, ssh_connection_id, location, specs, notes) =
            row;
        let server_type = server_type.parse().unwrap_or_default();
        let specs = specs.and_then(|s| serde_json::from_str(&s).ok());

        pctrl_core::Server {
            id,
            name,
            host,
            server_type,
            provider,
            ssh_connection_id,
            location,
            specs,
            notes,
        }
    }
}
