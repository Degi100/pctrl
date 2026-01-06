use pctrl_core::{CoolifyInstance, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Coolify deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub status: String,
    pub url: Option<String>,
}

/// Coolify manager
pub struct CoolifyManager {
    instances: Vec<CoolifyInstance>,
    client: Client,
}

impl CoolifyManager {
    pub fn new() -> Self {
        Self {
            instances: Vec::new(),
            client: Client::new(),
        }
    }

    /// Add a Coolify instance
    pub fn add_instance(&mut self, instance: CoolifyInstance) {
        self.instances.push(instance);
    }

    /// List deployments on an instance
    pub async fn list_deployments(&self, instance_id: &str) -> Result<Vec<Deployment>> {
        let instance = self
            .instances
            .iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| pctrl_core::Error::Coolify("Instance not found".to_string()))?;

        let url = format!("{}/api/v1/deployments", instance.url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", instance.api_key))
            .send()
            .await
            .map_err(|e| pctrl_core::Error::Coolify(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(pctrl_core::Error::Coolify(format!(
                "API request failed with status: {}",
                response.status()
            )));
        }

        let deployments: Vec<Deployment> = response
            .json()
            .await
            .map_err(|e| pctrl_core::Error::Coolify(format!("Failed to parse response: {}", e)))?;

        Ok(deployments)
    }

    /// Deploy a project
    pub async fn deploy_project(&self, instance_id: &str, project_id: &str) -> Result<()> {
        let instance = self
            .instances
            .iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| pctrl_core::Error::Coolify("Instance not found".to_string()))?;

        let url = format!("{}/api/v1/deployments", instance.url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", instance.api_key))
            .json(&serde_json::json!({ "project_id": project_id }))
            .send()
            .await
            .map_err(|e| pctrl_core::Error::Coolify(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(pctrl_core::Error::Coolify(format!(
                "Deployment failed with status: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// List all instances
    pub fn list_instances(&self) -> &[CoolifyInstance] {
        &self.instances
    }
}

impl Default for CoolifyManager {
    fn default() -> Self {
        Self::new()
    }
}
