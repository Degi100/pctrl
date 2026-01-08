use bollard::container::{ListContainersOptions, StartContainerOptions, StopContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::Docker;
use futures_util::StreamExt;
use pctrl_core::{DockerHost, Result};
use serde::{Deserialize, Serialize};

/// Container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
}

/// Docker manager
pub struct DockerManager {
    hosts: Vec<DockerHost>,
}

impl DockerManager {
    pub fn new() -> Self {
        Self { hosts: Vec::new() }
    }

    /// Add a Docker host
    pub fn add_host(&mut self, host: DockerHost) {
        self.hosts.push(host);
    }

    /// Connect to a Docker host
    fn connect(&self, id: &str) -> Result<Docker> {
        let host = self
            .hosts
            .iter()
            .find(|h| h.id == id)
            .ok_or_else(|| pctrl_core::Error::Docker("Host not found".to_string()))?;

        Docker::connect_with_socket(&host.url, 120, bollard::API_DEFAULT_VERSION)
            .map_err(|e| pctrl_core::Error::Docker(format!("Connection failed: {}", e)))
    }

    /// List containers on a host
    pub async fn list_containers(&self, host_id: &str) -> Result<Vec<ContainerInfo>> {
        let docker = self.connect(host_id)?;

        let containers = docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await
            .map_err(|e| pctrl_core::Error::Docker(format!("Failed to list containers: {}", e)))?;

        let mut result = Vec::new();
        for container in containers {
            result.push(ContainerInfo {
                id: container.id.unwrap_or_default(),
                name: container
                    .names
                    .unwrap_or_default()
                    .first()
                    .cloned()
                    .unwrap_or_default(),
                image: container.image.unwrap_or_default(),
                state: container.state.unwrap_or_default(),
                status: container.status.unwrap_or_default(),
            });
        }

        Ok(result)
    }

    /// Start a container
    pub async fn start_container(&self, host_id: &str, container_id: &str) -> Result<()> {
        let docker = self.connect(host_id)?;

        docker
            .start_container(container_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| pctrl_core::Error::Docker(format!("Failed to start container: {}", e)))?;

        Ok(())
    }

    /// Stop a container
    pub async fn stop_container(&self, host_id: &str, container_id: &str) -> Result<()> {
        let docker = self.connect(host_id)?;

        docker
            .stop_container(container_id, None::<StopContainerOptions>)
            .await
            .map_err(|e| pctrl_core::Error::Docker(format!("Failed to stop container: {}", e)))?;

        Ok(())
    }

    /// List all hosts
    pub fn list_hosts(&self) -> &[DockerHost] {
        &self.hosts
    }

    /// Get a host by ID
    pub fn get_host(&self, id: &str) -> Option<&DockerHost> {
        self.hosts.iter().find(|h| h.id == id)
    }

    /// Execute a command inside a container
    pub async fn exec_in_container(
        &self,
        host_id: &str,
        container_id: &str,
        command: &str,
    ) -> Result<String> {
        let docker = self.connect(host_id)?;

        // Parse command into args (simple split by whitespace)
        let cmd: Vec<&str> = command.split_whitespace().collect();

        // Create exec instance
        let exec = docker
            .create_exec(
                container_id,
                CreateExecOptions {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    cmd: Some(cmd),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| pctrl_core::Error::Docker(format!("Failed to create exec: {}", e)))?;

        // Start exec and collect output
        let output = docker
            .start_exec(&exec.id, None)
            .await
            .map_err(|e| pctrl_core::Error::Docker(format!("Failed to start exec: {}", e)))?;

        let mut result = String::new();
        if let StartExecResults::Attached { mut output, .. } = output {
            while let Some(chunk) = output.next().await {
                match chunk {
                    Ok(bollard::container::LogOutput::StdOut { message }) => {
                        result.push_str(&String::from_utf8_lossy(&message));
                    }
                    Ok(bollard::container::LogOutput::StdErr { message }) => {
                        result.push_str(&String::from_utf8_lossy(&message));
                    }
                    _ => {}
                }
            }
        }

        Ok(result)
    }

    /// Health check - verify connection to Docker host
    pub async fn health_check(&self, host_id: &str) -> Result<()> {
        let docker = self.connect(host_id)?;

        docker
            .ping()
            .await
            .map_err(|e| pctrl_core::Error::Docker(format!("Health check failed: {}", e)))?;

        Ok(())
    }
}

impl Default for DockerManager {
    fn default() -> Self {
        Self::new()
    }
}
