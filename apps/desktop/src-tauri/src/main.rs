// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pctrl_core::{
    DatabaseCredentials, DatabaseType, Domain, DomainType, Project, ProjectStatus, Script,
    ScriptType, Server, ServerType,
};
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
// v6 DTOs for Frontend Communication
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectDto {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub stack: Option<Vec<String>>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerDto {
    pub id: Option<String>,
    pub name: String,
    pub host: String,
    pub server_type: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DomainDto {
    pub id: Option<String>,
    pub domain: String,
    pub domain_type: Option<String>,
    pub ssl: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseCredentialsDto {
    pub id: Option<String>,
    pub name: String,
    pub db_type: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScriptDto {
    pub id: Option<String>,
    pub name: String,
    pub command: String,
    pub script_type: Option<String>,
    pub description: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────────────────────

fn get_db_path() -> String {
    // Use data_local_dir to match CLI (AppData\Local on Windows)
    if let Some(data_dir) = dirs::data_local_dir() {
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
// Project Commands (v6)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.list_projects().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_project(state: State<'_, AppState>, data: ProjectDto) -> Result<Project, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let status: ProjectStatus = data
        .status
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();

    let project = Project {
        id: data.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        name: data.name,
        description: data.description,
        stack: data.stack.unwrap_or_default(),
        status,
        color: None,
        icon: None,
        notes: None,
    };

    db.save_project(&project).await.map_err(|e| e.to_string())?;

    Ok(project)
}

#[tauri::command]
async fn delete_project(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.remove_project(&id).await.map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Server Commands (v6)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn list_servers(state: State<'_, AppState>) -> Result<Vec<Server>, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.list_servers().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_server(state: State<'_, AppState>, data: ServerDto) -> Result<Server, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let server_type: ServerType = data
        .server_type
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();

    let server = Server {
        id: data.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        name: data.name,
        host: data.host,
        server_type,
        provider: data.provider,
        ssh_connection_id: None,
        location: None,
        specs: None,
        notes: None,
    };

    db.save_server(&server).await.map_err(|e| e.to_string())?;

    Ok(server)
}

#[tauri::command]
async fn delete_server(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.remove_server(&id).await.map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Domain Commands (v6)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn list_domains(state: State<'_, AppState>) -> Result<Vec<Domain>, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.list_domains().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_domain(state: State<'_, AppState>, data: DomainDto) -> Result<Domain, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let domain_type: DomainType = data
        .domain_type
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();

    let domain = Domain {
        id: data.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        domain: data.domain,
        domain_type,
        ssl: data.ssl.unwrap_or(true),
        ssl_expiry: None,
        cloudflare_zone_id: None,
        cloudflare_record_id: None,
        server_id: None,
        container_id: None,
        notes: None,
    };

    db.save_domain(&domain).await.map_err(|e| e.to_string())?;

    Ok(domain)
}

#[tauri::command]
async fn delete_domain(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.remove_domain(&id).await.map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Database Credentials Commands (v6)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn list_databases(state: State<'_, AppState>) -> Result<Vec<DatabaseCredentials>, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.list_database_credentials()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_database(
    state: State<'_, AppState>,
    data: DatabaseCredentialsDto,
) -> Result<DatabaseCredentials, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let db_type: DatabaseType = data
        .db_type
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();

    let database = DatabaseCredentials {
        id: data.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        name: data.name,
        db_type,
        host: data.host,
        port: data.port,
        database_name: None,
        username: data.username,
        password: data.password,
        connection_string: None,
        server_id: None,
        container_id: None,
        notes: None,
    };

    db.save_database_credentials(&database)
        .await
        .map_err(|e| e.to_string())?;

    Ok(database)
}

#[tauri::command]
async fn delete_database(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.remove_database_credentials(&id)
        .await
        .map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Script Commands (v6)
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn list_scripts(state: State<'_, AppState>) -> Result<Vec<Script>, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.list_scripts().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_script(state: State<'_, AppState>, data: ScriptDto) -> Result<Script, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let script_type: ScriptType = data
        .script_type
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();

    let script = Script {
        id: data.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        name: data.name,
        description: data.description,
        command: data.command,
        script_type,
        server_id: None,
        project_id: None,
        docker_host_id: None,
        container_id: None,
        dangerous: false,
        last_run: None,
        last_result: None,
        exit_code: None,
        last_output: None,
    };

    db.save_script(&script).await.map_err(|e| e.to_string())?;

    Ok(script)
}

#[tauri::command]
async fn delete_script(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.remove_script(&id).await.map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Legacy Count Command (for migration warning)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct LegacyCounts {
    pub ssh: usize,
    pub docker: usize,
    pub coolify: usize,
    pub git: usize,
    pub total: usize,
}

#[tauri::command]
async fn get_legacy_counts(state: State<'_, AppState>) -> Result<LegacyCounts, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let config = db.load_config().await.map_err(|e| e.to_string())?;

    let counts = LegacyCounts {
        ssh: config.ssh_connections.len(),
        docker: config.docker_hosts.len(),
        coolify: config.coolify_instances.len(),
        git: config.git_repos.len(),
        total: config.ssh_connections.len()
            + config.docker_hosts.len()
            + config.coolify_instances.len()
            + config.git_repos.len(),
    };

    Ok(counts)
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
            // v6 Commands
            list_projects,
            add_project,
            delete_project,
            list_servers,
            add_server,
            delete_server,
            list_domains,
            add_domain,
            delete_domain,
            list_databases,
            add_database,
            delete_database,
            list_scripts,
            add_script,
            delete_script,
            get_legacy_counts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
