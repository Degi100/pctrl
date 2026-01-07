// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pctrl_core::{AuthMethod, Config, CoolifyInstance, DockerHost, GitRepo, SshConnection};
use pctrl_database::Database;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// App State
// ─────────────────────────────────────────────────────────────────────────────

struct AppState {
    db: Arc<Mutex<Option<Database>>>,
}

// ─────────────────────────────────────────────────────────────────────────────
// DTOs for Frontend Communication
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct SshConnectionDto {
    pub id: Option<String>,
    pub name: String,
    pub host: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerHostDto {
    pub id: Option<String>,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoolifyInstanceDto {
    pub id: Option<String>,
    pub name: String,
    pub url: String,
    pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitRepoDto {
    pub id: Option<String>,
    pub name: String,
    pub path: String,
    pub remote_url: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────────────────────

fn get_db_path() -> String {
    if let Some(data_dir) = dirs::data_dir() {
        let pctrl_dir = data_dir.join("pctrl");
        std::fs::create_dir_all(&pctrl_dir).ok();
        pctrl_dir.join("pctrl.db").to_string_lossy().to_string()
    } else {
        "pctrl.db".to_string()
    }
}

async fn ensure_db(state: &State<'_, AppState>) -> Result<(), String> {
    let mut db_guard = state.db.lock().await;
    if db_guard.is_none() {
        let db_path = get_db_path();
        let db = Database::new(&db_path, None)
            .await
            .map_err(|e| e.to_string())?;
        *db_guard = Some(db);
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Config Commands
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<Config, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.load_config().await.map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// SSH Commands
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn add_ssh(state: State<'_, AppState>, data: SshConnectionDto) -> Result<SshConnection, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let conn = SshConnection {
        id: data.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        name: data.name,
        host: data.host,
        port: data.port.unwrap_or(22),
        username: data.username.unwrap_or_else(|| "root".to_string()),
        auth_method: AuthMethod::PublicKey {
            key_path: data.key_path.unwrap_or_else(|| "~/.ssh/id_rsa".to_string()),
        },
    };

    db.save_ssh_connection(&conn)
        .await
        .map_err(|e| e.to_string())?;

    Ok(conn)
}

#[tauri::command]
async fn update_ssh(state: State<'_, AppState>, data: SshConnectionDto) -> Result<SshConnection, String> {
    let id = data.id.clone().ok_or("ID is required for update")?;

    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    if !db.ssh_connection_exists(&id).await.map_err(|e| e.to_string())? {
        return Err(format!("SSH connection with id '{}' not found", id));
    }

    let conn = SshConnection {
        id,
        name: data.name,
        host: data.host,
        port: data.port.unwrap_or(22),
        username: data.username.unwrap_or_else(|| "root".to_string()),
        auth_method: AuthMethod::PublicKey {
            key_path: data.key_path.unwrap_or_else(|| "~/.ssh/id_rsa".to_string()),
        },
    };

    db.save_ssh_connection(&conn)
        .await
        .map_err(|e| e.to_string())?;

    Ok(conn)
}

#[tauri::command]
async fn delete_ssh(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.remove_ssh_connection(&id)
        .await
        .map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Docker Commands
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn add_docker(state: State<'_, AppState>, data: DockerHostDto) -> Result<DockerHost, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let host = DockerHost {
        id: data.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        name: data.name,
        url: data.url.unwrap_or_else(|| "unix:///var/run/docker.sock".to_string()),
    };

    db.save_docker_host(&host)
        .await
        .map_err(|e| e.to_string())?;

    Ok(host)
}

#[tauri::command]
async fn update_docker(state: State<'_, AppState>, data: DockerHostDto) -> Result<DockerHost, String> {
    let id = data.id.clone().ok_or("ID is required for update")?;

    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    if !db.docker_host_exists(&id).await.map_err(|e| e.to_string())? {
        return Err(format!("Docker host with id '{}' not found", id));
    }

    let host = DockerHost {
        id,
        name: data.name,
        url: data.url.unwrap_or_else(|| "unix:///var/run/docker.sock".to_string()),
    };

    db.save_docker_host(&host)
        .await
        .map_err(|e| e.to_string())?;

    Ok(host)
}

#[tauri::command]
async fn delete_docker(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.remove_docker_host(&id)
        .await
        .map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Coolify Commands
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn add_coolify(state: State<'_, AppState>, data: CoolifyInstanceDto) -> Result<CoolifyInstance, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let instance = CoolifyInstance {
        id: data.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        name: data.name,
        url: data.url,
        api_key: data.api_key,
    };

    db.save_coolify_instance(&instance)
        .await
        .map_err(|e| e.to_string())?;

    Ok(instance)
}

#[tauri::command]
async fn update_coolify(state: State<'_, AppState>, data: CoolifyInstanceDto) -> Result<CoolifyInstance, String> {
    let id = data.id.clone().ok_or("ID is required for update")?;

    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    if !db.coolify_instance_exists(&id).await.map_err(|e| e.to_string())? {
        return Err(format!("Coolify instance with id '{}' not found", id));
    }

    let instance = CoolifyInstance {
        id,
        name: data.name,
        url: data.url,
        api_key: data.api_key,
    };

    db.save_coolify_instance(&instance)
        .await
        .map_err(|e| e.to_string())?;

    Ok(instance)
}

#[tauri::command]
async fn delete_coolify(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.remove_coolify_instance(&id)
        .await
        .map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Git Commands
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn add_git(state: State<'_, AppState>, data: GitRepoDto) -> Result<GitRepo, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let repo = GitRepo {
        id: data.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        name: data.name,
        path: data.path,
        remote_url: data.remote_url,
    };

    db.save_git_repo(&repo)
        .await
        .map_err(|e| e.to_string())?;

    Ok(repo)
}

#[tauri::command]
async fn update_git(state: State<'_, AppState>, data: GitRepoDto) -> Result<GitRepo, String> {
    let id = data.id.clone().ok_or("ID is required for update")?;

    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    if !db.git_repo_exists(&id).await.map_err(|e| e.to_string())? {
        return Err(format!("Git repository with id '{}' not found", id));
    }

    let repo = GitRepo {
        id,
        name: data.name,
        path: data.path,
        remote_url: data.remote_url,
    };

    db.save_git_repo(&repo)
        .await
        .map_err(|e| e.to_string())?;

    Ok(repo)
}

#[tauri::command]
async fn delete_git(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.remove_git_repo(&id)
        .await
        .map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            db: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            // SSH
            add_ssh,
            update_ssh,
            delete_ssh,
            // Docker
            add_docker,
            update_docker,
            delete_docker,
            // Coolify
            add_coolify,
            update_coolify,
            delete_coolify,
            // Git
            add_git,
            update_git,
            delete_git,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
