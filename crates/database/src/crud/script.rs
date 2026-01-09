//! Script CRUD operations

use crate::Database;
use pctrl_core::Result;

impl Database {
    /// Save a script
    pub async fn save_script(&self, script: &pctrl_core::Script) -> Result<()> {
        let last_result = script.last_result.as_ref().map(|r| r.to_string());

        sqlx::query(
            "INSERT OR REPLACE INTO scripts (id, name, description, command, script_type, server_id, project_id, docker_host_id, container_id, dangerous, last_run, last_result, exit_code, last_output)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
        .bind(script.exit_code)
        .bind(&script.last_output)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Get a script by ID
    pub async fn get_script(&self, id: &str) -> Result<Option<pctrl_core::Script>> {
        let row: Option<ScriptRow> = sqlx::query_as(
            "SELECT id, name, description, command, script_type, server_id, project_id, docker_host_id, container_id, dangerous, last_run, last_result, exit_code, last_output FROM scripts WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(Self::row_to_script))
    }

    /// List all scripts
    pub async fn list_scripts(&self) -> Result<Vec<pctrl_core::Script>> {
        let rows: Vec<ScriptRow> = sqlx::query_as(
            "SELECT id, name, description, command, script_type, server_id, project_id, docker_host_id, container_id, dangerous, last_run, last_result, exit_code, last_output FROM scripts ORDER BY name",
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
        let rows: Vec<ScriptRow> = sqlx::query_as(
            "SELECT id, name, description, command, script_type, server_id, project_id, docker_host_id, container_id, dangerous, last_run, last_result, exit_code, last_output FROM scripts WHERE project_id = ? ORDER BY name",
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
        exit_code: Option<i32>,
        output: Option<&str>,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let result_str = result.to_string();
        // Truncate output to 10KB max
        let truncated_output = output.map(|o| {
            if o.len() > 10240 {
                format!("{}...[truncated]", &o[..10240])
            } else {
                o.to_string()
            }
        });

        sqlx::query(
            "UPDATE scripts SET last_run = ?, last_result = ?, exit_code = ?, last_output = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(&result_str)
        .bind(exit_code)
        .bind(&truncated_output)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Helper to convert a row tuple to Script
    fn row_to_script(row: ScriptRow) -> pctrl_core::Script {
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
            exit_code,
            last_output,
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
            exit_code,
            last_output,
        }
    }
}

/// Type alias for script row tuple
type ScriptRow = (
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
    Option<i32>,
    Option<String>,
);
