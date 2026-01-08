//! Database Credentials CRUD operations

use crate::Database;
use pctrl_core::Result;

impl Database {
    /// Save database credentials
    pub async fn save_database_credentials(
        &self,
        db_creds: &pctrl_core::DatabaseCredentials,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO databases (id, name, db_type, host, port, database_name, username, password, connection_string, server_id, container_id, notes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&db_creds.id)
        .bind(&db_creds.name)
        .bind(db_creds.db_type.to_string())
        .bind(&db_creds.host)
        .bind(db_creds.port.map(|p| p as i64))
        .bind(&db_creds.database_name)
        .bind(&db_creds.username)
        .bind(&db_creds.password)
        .bind(&db_creds.connection_string)
        .bind(&db_creds.server_id)
        .bind(&db_creds.container_id)
        .bind(&db_creds.notes)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Get database credentials by ID
    pub async fn get_database_credentials(
        &self,
        id: &str,
    ) -> Result<Option<pctrl_core::DatabaseCredentials>> {
        let row: Option<(
            String,
            String,
            String,
            Option<String>,
            Option<i64>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, db_type, host, port, database_name, username, password, connection_string, server_id, container_id, notes FROM databases WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(Self::row_to_database_credentials))
    }

    /// Get database credentials by name (case-insensitive)
    pub async fn get_database_credentials_by_name(
        &self,
        name: &str,
    ) -> Result<Option<pctrl_core::DatabaseCredentials>> {
        let row: Option<(
            String,
            String,
            String,
            Option<String>,
            Option<i64>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, db_type, host, port, database_name, username, password, connection_string, server_id, container_id, notes FROM databases WHERE LOWER(name) = LOWER(?)",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(Self::row_to_database_credentials))
    }

    /// List all database credentials
    pub async fn list_database_credentials(&self) -> Result<Vec<pctrl_core::DatabaseCredentials>> {
        let rows: Vec<(
            String,
            String,
            String,
            Option<String>,
            Option<i64>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, db_type, host, port, database_name, username, password, connection_string, server_id, container_id, notes FROM databases ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(Self::row_to_database_credentials)
            .collect())
    }

    /// Remove database credentials by ID
    pub async fn remove_database_credentials(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM databases WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Helper to convert a row tuple to DatabaseCredentials
    fn row_to_database_credentials(
        row: (
            String,
            String,
            String,
            Option<String>,
            Option<i64>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    ) -> pctrl_core::DatabaseCredentials {
        let (
            id,
            name,
            db_type,
            host,
            port,
            database_name,
            username,
            password,
            connection_string,
            server_id,
            container_id,
            notes,
        ) = row;
        let db_type = db_type.parse().unwrap_or_default();

        pctrl_core::DatabaseCredentials {
            id,
            name,
            db_type,
            host,
            port: port.map(|p| p as u16),
            database_name,
            username,
            password,
            connection_string,
            server_id,
            container_id,
            notes,
        }
    }
}
