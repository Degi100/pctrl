use chrono::{TimeZone, Utc};
use git2::Repository;
use pctrl_core::{GitRepo, Result};
use serde::{Deserialize, Serialize};

/// Release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub name: String,
    pub tag: String,
    pub message: String,
    pub date: String,
}

/// Format a Unix timestamp as a readable date string
fn format_timestamp(seconds: i64) -> String {
    match Utc.timestamp_opt(seconds, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        _ => String::new(),
    }
}

/// Git manager
pub struct GitManager {
    repos: Vec<GitRepo>,
}

impl GitManager {
    pub fn new() -> Self {
        Self { repos: Vec::new() }
    }

    /// Add a Git repository
    pub fn add_repo(&mut self, repo: GitRepo) {
        self.repos.push(repo);
    }

    /// Open a repository
    fn open_repo(&self, id: &str) -> Result<Repository> {
        let repo = self
            .repos
            .iter()
            .find(|r| r.id == id)
            .ok_or_else(|| pctrl_core::Error::Git("Repository not found".to_string()))?;

        Repository::open(&repo.path)
            .map_err(|e| pctrl_core::Error::Git(format!("Failed to open repository: {}", e)))
    }

    /// List tags/releases in a repository
    pub fn list_releases(&self, repo_id: &str) -> Result<Vec<Release>> {
        let repo = self.open_repo(repo_id)?;
        let mut releases = Vec::new();

        let tags = repo
            .tag_names(None)
            .map_err(|e| pctrl_core::Error::Git(format!("Failed to get tags: {}", e)))?;

        for tag_name in tags.iter().flatten() {
            if let Ok(obj) = repo.revparse_single(tag_name) {
                let tag = obj.peel_to_tag().ok();
                let message = tag
                    .as_ref()
                    .and_then(|t| t.message())
                    .unwrap_or("")
                    .to_string();

                // Get date: try annotated tag date first, then fall back to commit date
                let date = {
                    // First try: get tagger date from annotated tag
                    let tag_timestamp = tag
                        .as_ref()
                        .and_then(|t| t.tagger())
                        .map(|sig| sig.when().seconds());

                    // Second try: get commit date by peeling to commit
                    let commit_timestamp = obj.peel_to_commit().ok().map(|c| c.time().seconds());

                    // Use tag timestamp if available, otherwise commit timestamp
                    let timestamp = tag_timestamp.or(commit_timestamp);

                    // Format the timestamp as a readable date
                    match timestamp {
                        Some(secs) => format_timestamp(secs),
                        None => String::new(),
                    }
                };

                releases.push(Release {
                    name: tag_name.to_string(),
                    tag: tag_name.to_string(),
                    message,
                    date,
                });
            }
        }

        Ok(releases)
    }

    /// Create a new release/tag
    pub fn create_release(&self, repo_id: &str, tag_name: &str, message: &str) -> Result<()> {
        let repo = self.open_repo(repo_id)?;

        let signature = repo
            .signature()
            .map_err(|e| pctrl_core::Error::Git(format!("Failed to get signature: {}", e)))?;

        let head = repo
            .head()
            .map_err(|e| pctrl_core::Error::Git(format!("Failed to get HEAD: {}", e)))?;

        let target = head
            .peel_to_commit()
            .map_err(|e| pctrl_core::Error::Git(format!("Failed to get commit: {}", e)))?;

        repo.tag(tag_name, target.as_object(), &signature, message, false)
            .map_err(|e| pctrl_core::Error::Git(format!("Failed to create tag: {}", e)))?;

        Ok(())
    }

    /// Push tags to remote
    pub fn push_tags(&self, repo_id: &str) -> Result<()> {
        let repo = self.open_repo(repo_id)?;

        let mut remote = repo
            .find_remote("origin")
            .map_err(|e| pctrl_core::Error::Git(format!("Failed to find remote: {}", e)))?;

        remote
            .push(&["refs/tags/*:refs/tags/*"], None)
            .map_err(|e| pctrl_core::Error::Git(format!("Failed to push tags: {}", e)))?;

        Ok(())
    }

    /// List all repositories
    pub fn list_repos(&self) -> &[GitRepo] {
        &self.repos
    }
}

impl Default for GitManager {
    fn default() -> Self {
        Self::new()
    }
}
