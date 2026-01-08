//! Git Repository CRUD operations

use crate::Database;
use pctrl_core::Result;
use sqlx::Row;

impl Database {
    /// Add or update a Git repository
    pub async fn save_git_repo(&self, repo: &pctrl_core::GitRepo) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO git_repos (id, name, path, remote_url)
             VALUES (?, ?, ?, ?)",
        )
        .bind(&repo.id)
        .bind(&repo.name)
        .bind(&repo.path)
        .bind(&repo.remote_url)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Remove a Git repository by ID
    pub async fn remove_git_repo(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM git_repos WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if a Git repository exists
    pub async fn git_repo_exists(&self, id: &str) -> Result<bool> {
        let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM git_repos WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(|(count,)| count > 0).unwrap_or(false))
    }

    /// Load all Git repositories
    pub(crate) async fn load_git_repos(&self) -> Result<Vec<pctrl_core::GitRepo>> {
        let rows = sqlx::query("SELECT id, name, path, remote_url FROM git_repos")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let repos = rows
            .into_iter()
            .map(|row| pctrl_core::GitRepo {
                id: row.get("id"),
                name: row.get("name"),
                path: row.get("path"),
                remote_url: row.get("remote_url"),
            })
            .collect();

        Ok(repos)
    }
}
