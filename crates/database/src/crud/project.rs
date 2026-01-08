//! Project CRUD operations

use crate::Database;
use pctrl_core::Result;

impl Database {
    /// Save a project
    pub async fn save_project(&self, project: &pctrl_core::Project) -> Result<()> {
        let stack = serde_json::to_string(&project.stack)
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        sqlx::query(
            "INSERT OR REPLACE INTO projects (id, name, description, stack, status, color, icon, notes, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
        )
        .bind(&project.id)
        .bind(&project.name)
        .bind(&project.description)
        .bind(&stack)
        .bind(project.status.to_string())
        .bind(&project.color)
        .bind(&project.icon)
        .bind(&project.notes)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Get a project by ID
    pub async fn get_project(&self, id: &str) -> Result<Option<pctrl_core::Project>> {
        let row: Option<(
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, description, stack, status, color, icon, notes FROM projects WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(Self::row_to_project))
    }

    /// Get a project by name (case-insensitive)
    pub async fn get_project_by_name(&self, name: &str) -> Result<Option<pctrl_core::Project>> {
        let row: Option<(
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, description, stack, status, color, icon, notes FROM projects WHERE LOWER(name) = LOWER(?)",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(Self::row_to_project))
    }

    /// List all projects
    pub async fn list_projects(&self) -> Result<Vec<pctrl_core::Project>> {
        let rows: Vec<(
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, name, description, stack, status, color, icon, notes FROM projects ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(rows.into_iter().map(Self::row_to_project).collect())
    }

    /// Remove a project by ID
    pub async fn remove_project(&self, id: &str) -> Result<bool> {
        // Also remove all project_resources for this project
        sqlx::query("DELETE FROM project_resources WHERE project_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let result = sqlx::query("DELETE FROM projects WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if a project exists
    pub async fn project_exists(&self, id: &str) -> Result<bool> {
        let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM projects WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(|(count,)| count > 0).unwrap_or(false))
    }

    /// Helper to convert a row tuple to Project
    fn row_to_project(
        row: (
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    ) -> pctrl_core::Project {
        let (id, name, description, stack, status, color, icon, notes) = row;
        let stack: Vec<String> = stack
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        let status = status.parse().unwrap_or_default();

        pctrl_core::Project {
            id,
            name,
            description,
            stack,
            status,
            color,
            icon,
            notes,
        }
    }
}
