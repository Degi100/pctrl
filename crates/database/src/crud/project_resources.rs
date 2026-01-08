//! Project Resource linking operations

use crate::Database;
use pctrl_core::Result;

impl Database {
    /// Link a resource to a project
    pub async fn link_project_resource(
        &self,
        resource: &pctrl_core::ProjectResource,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO project_resources (id, project_id, resource_type, resource_id, role, notes)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&resource.id)
        .bind(&resource.project_id)
        .bind(resource.resource_type.to_string())
        .bind(&resource.resource_id)
        .bind(&resource.role)
        .bind(&resource.notes)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Get all resources for a project
    pub async fn get_project_resources(
        &self,
        project_id: &str,
    ) -> Result<Vec<pctrl_core::ProjectResource>> {
        let rows: Vec<(String, String, String, String, Option<String>, Option<String>)> =
            sqlx::query_as(
                "SELECT id, project_id, resource_type, resource_id, role, notes FROM project_resources WHERE project_id = ?",
            )
            .bind(project_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let resources = rows
            .into_iter()
            .map(
                |(id, project_id, resource_type, resource_id, role, notes)| {
                    let resource_type = resource_type
                        .parse()
                        .unwrap_or(pctrl_core::ResourceType::Server);

                    pctrl_core::ProjectResource {
                        id,
                        project_id,
                        resource_type,
                        resource_id,
                        role,
                        notes,
                    }
                },
            )
            .collect();

        Ok(resources)
    }

    /// Unlink a resource from a project
    pub async fn unlink_project_resource(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM project_resources WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Get projects that have a specific resource linked
    pub async fn get_projects_for_resource(
        &self,
        resource_type: &pctrl_core::ResourceType,
        resource_id: &str,
    ) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT project_id FROM project_resources WHERE resource_type = ? AND resource_id = ?",
        )
        .bind(resource_type.to_string())
        .bind(resource_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }
}
