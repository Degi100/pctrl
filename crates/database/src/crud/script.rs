//! Script CRUD operations

use crate::Database;
use pctrl_core::Result;

impl Database {
    /// Save a script
    pub async fn save_script(&self, script: &pctrl_core::Script) -> Result<()> {
        let last_result = script.last_result.as_ref().map(|r| r.to_string());

        sqlx::query(
            "INSERT OR REPLACE INTO scripts (id, name, description, command, script_type, server_id, project_id, docker_host_id, container_id, dangerous, last_run, last_result)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&script.id)
        .bind(&script.name)
        .bind(&script.description)
        .bind(&script.command)
        .bind(script.script_type.to_string())
        .bind(&script.server_id)
        .bind(&script.project_id)
        .bind(&script.docker_host_id)
        .bind(&script.container_id)
        .bind(script.dangerous)
        .bind(&script.last_run)
        .bind(&last_result)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Get a script by ID
    pub async fn get_script(&self, id: &str) -> Result<Option<pctrl_core::Script>> {
        let row: Option<(
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            bool,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, description, command, script_type, server_id, project_id, docker_host_id, container_id, dangerous, last_run, last_result FROM scripts WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(Self::row_to_script))
    }

    /// List all scripts
    pub async fn list_scripts(&self) -> Result<Vec<pctrl_core::Script>> {
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            bool,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, description, command, script_type, server_id, project_id, docker_host_id, container_id, dangerous, last_run, last_result FROM scripts ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(rows.into_iter().map(Self::row_to_script).collect())
    }

    /// List scripts for a project
    pub async fn list_scripts_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<pctrl_core::Script>> {
        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            bool,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, description, command, script_type, server_id, project_id, docker_host_id, container_id, dangerous, last_run, last_result FROM scripts WHERE project_id = ? ORDER BY name",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(rows.into_iter().map(Self::row_to_script).collect())
    }

    /// Remove a script by ID
    pub async fn remove_script(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM scripts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Update script execution result
    pub async fn update_script_result(
        &self,
        id: &str,
        result: pctrl_core::ScriptResult,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let result_str = result.to_string();

        sqlx::query("UPDATE scripts SET last_run = ?, last_result = ? WHERE id = ?")
            .bind(&now)
            .bind(&result_str)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Helper to convert a row tuple to Script
    fn row_to_script(
        row: (
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            bool,
            Option<String>,
            Option<String>,
        ),
    ) -> pctrl_core::Script {
        let (
            id,
            name,
            description,
            command,
            script_type,
            server_id,
            project_id,
            docker_host_id,
            container_id,
            dangerous,
            last_run,
            last_result,
        ) = row;

        let script_type = script_type.parse().unwrap_or_default();
        let last_result = last_result.and_then(|r| match r.as_str() {
            "success" => Some(pctrl_core::ScriptResult::Success),
            "error" => Some(pctrl_core::ScriptResult::Error),
            _ => None,
        });

        pctrl_core::Script {
            id,
            name,
            description,
            command,
            script_type,
            server_id,
            project_id,
            docker_host_id,
            container_id,
            dangerous,
            last_run,
            last_result,
        }
    }
}
