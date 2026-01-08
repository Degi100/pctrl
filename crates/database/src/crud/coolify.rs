//! Coolify Instance CRUD operations

use crate::Database;
use pctrl_core::Result;
use sqlx::Row;

impl Database {
    /// Add or update a Coolify instance
    pub async fn save_coolify_instance(
        &self,
        instance: &pctrl_core::CoolifyInstance,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO coolify_instances (id, name, url, api_key)
             VALUES (?, ?, ?, ?)",
        )
        .bind(&instance.id)
        .bind(&instance.name)
        .bind(&instance.url)
        .bind(&instance.api_key)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Remove a Coolify instance by ID
    pub async fn remove_coolify_instance(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM coolify_instances WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if a Coolify instance exists
    pub async fn coolify_instance_exists(&self, id: &str) -> Result<bool> {
        let row: Option<(i64,)> =
            sqlx::query_as("SELECT COUNT(*) FROM coolify_instances WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(|(count,)| count > 0).unwrap_or(false))
    }

    /// Load all Coolify instances
    pub(crate) async fn load_coolify_instances(&self) -> Result<Vec<pctrl_core::CoolifyInstance>> {
        let rows = sqlx::query("SELECT id, name, url, api_key FROM coolify_instances")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let instances = rows
            .into_iter()
            .map(|row| pctrl_core::CoolifyInstance {
                id: row.get("id"),
                name: row.get("name"),
                url: row.get("url"),
                api_key: row.get("api_key"),
            })
            .collect();

        Ok(instances)
    }
}
