//! SSH Connection CRUD operations

use crate::Database;
use pctrl_core::Result;
use sqlx::Row;

impl Database {
    /// Add or update a single SSH connection
    pub async fn save_ssh_connection(&self, conn: &pctrl_core::SshConnection) -> Result<()> {
        let auth_method = serde_json::to_string(&conn.auth_method)
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        sqlx::query(
            "INSERT OR REPLACE INTO ssh_connections (id, name, host, port, username, auth_method)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&conn.id)
        .bind(&conn.name)
        .bind(&conn.host)
        .bind(conn.port as i64)
        .bind(&conn.username)
        .bind(&auth_method)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Remove an SSH connection by ID
    pub async fn remove_ssh_connection(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM ssh_connections WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if an SSH connection exists
    pub async fn ssh_connection_exists(&self, id: &str) -> Result<bool> {
        let row: Option<(i64,)> =
            sqlx::query_as("SELECT COUNT(*) FROM ssh_connections WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(|(count,)| count > 0).unwrap_or(false))
    }

    /// Get an SSH connection by ID
    pub async fn get_ssh_connection(&self, id: &str) -> Result<Option<pctrl_core::SshConnection>> {
        let row = sqlx::query(
            "SELECT id, name, host, port, username, auth_method FROM ssh_connections WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        match row {
            Some(row) => {
                let auth_method: String = row.get("auth_method");
                let auth_method = serde_json::from_str(&auth_method)
                    .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

                Ok(Some(pctrl_core::SshConnection {
                    id: row.get("id"),
                    name: row.get("name"),
                    host: row.get("host"),
                    port: row.get::<i64, _>("port") as u16,
                    username: row.get("username"),
                    auth_method,
                }))
            }
            None => Ok(None),
        }
    }

    /// Load all SSH connections
    pub(crate) async fn load_ssh_connections(&self) -> Result<Vec<pctrl_core::SshConnection>> {
        let rows =
            sqlx::query("SELECT id, name, host, port, username, auth_method FROM ssh_connections")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let mut connections = Vec::new();
        for row in rows {
            let auth_method: String = row.get("auth_method");
            let auth_method = serde_json::from_str(&auth_method)
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

            connections.push(pctrl_core::SshConnection {
                id: row.get("id"),
                name: row.get("name"),
                host: row.get("host"),
                port: row.get::<i64, _>("port") as u16,
                username: row.get("username"),
                auth_method,
            });
        }

        Ok(connections)
    }
}
