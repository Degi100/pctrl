//! Connection health check functions

use super::types::ConnectionStatus;
use pctrl_coolify::CoolifyManager;
use pctrl_core::{CoolifyInstance, DockerHost, SshConnection};
use pctrl_docker::DockerManager;
use pctrl_ssh::SshManager;
use std::time::Duration;

/// Check SSH connections using spawn_blocking (ssh2 is synchronous)
pub async fn check_ssh_connections(
    connections: Vec<SshConnection>,
) -> Vec<(String, ConnectionStatus)> {
    let mut results = Vec::new();

    for conn in connections {
        let conn_clone = conn.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut ssh_manager = SshManager::new();
            ssh_manager.add_connection(conn_clone.clone());

            match ssh_manager.test_connection(&conn_clone.id, None) {
                Ok(_) => ConnectionStatus::Online,
                Err(_) => ConnectionStatus::Offline,
            }
        });

        match tokio::time::timeout(Duration::from_secs(5), result).await {
            Ok(Ok(status)) => results.push((conn.id.clone(), status)),
            _ => results.push((conn.id.clone(), ConnectionStatus::Offline)),
        }
    }

    results
}

/// Check Docker connections (async)
pub async fn check_docker_connections(hosts: Vec<DockerHost>) -> Vec<(String, ConnectionStatus)> {
    let mut results = Vec::new();

    for host in hosts {
        let mut docker_manager = DockerManager::new();
        docker_manager.add_host(host.clone());

        let check = docker_manager.health_check(&host.id);
        match tokio::time::timeout(Duration::from_secs(5), check).await {
            Ok(Ok(_)) => results.push((host.id.clone(), ConnectionStatus::Online)),
            _ => results.push((host.id.clone(), ConnectionStatus::Offline)),
        }
    }

    results
}

/// Check Coolify connections (async)
pub async fn check_coolify_connections(
    instances: Vec<CoolifyInstance>,
) -> Vec<(String, ConnectionStatus)> {
    let mut results = Vec::new();

    for instance in instances {
        let mut coolify_manager = CoolifyManager::new();
        coolify_manager.add_instance(instance.clone());

        let check = coolify_manager.health_check(&instance.id);
        match tokio::time::timeout(Duration::from_secs(5), check).await {
            Ok(Ok(_)) => results.push((instance.id.clone(), ConnectionStatus::Online)),
            _ => results.push((instance.id.clone(), ConnectionStatus::Offline)),
        }
    }

    results
}
