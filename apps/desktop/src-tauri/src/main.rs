// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pctrl_core::{
    AuthMethod, Credential, CredentialData, CredentialType, DatabaseCredentials, DatabaseType,
    Domain, DomainType, Project, ProjectStatus, Script, ScriptType, Server, ServerType,
    SshConnection,
};
use pctrl_database::Database;
use pctrl_ssh::SshManager;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialDto {
    pub id: Option<String>,
    pub name: String,
    pub credential_type: String,
    pub username: Option<String>,
    pub port: Option<u16>,
    pub key_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStatusDto {
    pub online: bool,
    pub uptime: Option<String>,
    pub load: Option<String>,
    pub memory: Option<String>,
    pub disk: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerWithCredentialDto {
    pub id: Option<String>,
    pub name: String,
    pub host: String,
    pub server_type: Option<String>,
    pub provider: Option<String>,
    pub credential_id: Option<String>,
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
async fn add_server(
    state: State<'_, AppState>,
    data: ServerWithCredentialDto,
) -> Result<Server, String> {
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
        credential_id: data.credential_id,
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
// Credential Commands
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn list_credentials(state: State<'_, AppState>) -> Result<Vec<Credential>, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.list_credentials().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_credential(
    state: State<'_, AppState>,
    data: CredentialDto,
) -> Result<Credential, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let credential_type: CredentialType = data.credential_type.parse().map_err(|e: String| e)?;

    let cred_data = match credential_type {
        CredentialType::SshKey => {
            let username = data.username.ok_or("SSH requires username")?;
            let key_path = data.key_path.ok_or("SSH requires key_path")?;
            CredentialData::SshKey {
                username,
                port: data.port.unwrap_or(22),
                key_path,
                passphrase: None,
            }
        }
        CredentialType::SshAgent => {
            let username = data.username.ok_or("SSH Agent requires username")?;
            CredentialData::SshAgent {
                username,
                port: data.port.unwrap_or(22),
            }
        }
        _ => return Err("Unsupported credential type for desktop".to_string()),
    };

    let credential = Credential {
        id: data.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        name: data.name,
        credential_type,
        data: cred_data,
        notes: None,
    };

    db.save_credential(&credential)
        .await
        .map_err(|e| e.to_string())?;

    Ok(credential)
}

#[tauri::command]
async fn delete_credential(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.remove_credential(&id).await.map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Server SSH Commands
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn get_server_status(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<ServerStatusDto, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    // Get server
    let server = db
        .get_server(&server_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Server not found")?;

    // Check if credential is configured
    let cred_id = match &server.credential_id {
        Some(id) => id,
        None => {
            return Ok(ServerStatusDto {
                online: false,
                uptime: None,
                load: None,
                memory: None,
                disk: None,
                error: Some("No credential configured".to_string()),
            });
        }
    };

    // Get credential
    let credential = db
        .get_credential(cred_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Credential not found")?;

    // Create SSH connection
    let (username, port, auth_method) = match &credential.data {
        CredentialData::SshKey {
            username,
            port,
            key_path,
            passphrase,
        } => (
            username.clone(),
            *port,
            AuthMethod::Key {
                path: key_path.clone(),
                passphrase: passphrase.clone(),
            },
        ),
        CredentialData::SshAgent { username, port } => (username.clone(), *port, AuthMethod::Agent),
        _ => {
            return Ok(ServerStatusDto {
                online: false,
                uptime: None,
                load: None,
                memory: None,
                disk: None,
                error: Some("Credential is not SSH type".to_string()),
            });
        }
    };

    let ssh_conn = SshConnection {
        id: credential.id.clone(),
        name: credential.name.clone(),
        host: server.host.clone(),
        port,
        username,
        auth_method,
    };

    let mut ssh_manager = SshManager::new();
    ssh_manager.add_connection(ssh_conn);
    let conn_id = credential.id.clone();

    // Run status commands in blocking task
    let result = tokio::task::spawn_blocking(move || {
        let mut status = ServerStatusDto {
            online: false,
            uptime: None,
            load: None,
            memory: None,
            disk: None,
            error: None,
        };

        // Try to get uptime (tests connection)
        match ssh_manager.execute_command(&conn_id, "uptime -p 2>/dev/null || uptime") {
            Ok(output) => {
                status.online = true;
                status.uptime = Some(output.trim().to_string());
            }
            Err(e) => {
                status.error = Some(e.to_string());
                return status;
            }
        }

        // Get load
        if let Ok(output) =
            ssh_manager.execute_command(&conn_id, "cat /proc/loadavg | cut -d' ' -f1-3")
        {
            status.load = Some(output.trim().to_string());
        }

        // Get memory
        if let Ok(output) =
            ssh_manager.execute_command(&conn_id, "free -h | grep Mem | awk '{print $3 \"/\" $2}'")
        {
            status.memory = Some(output.trim().to_string());
        }

        // Get disk
        if let Ok(output) = ssh_manager.execute_command(
            &conn_id,
            "df -h / | tail -1 | awk '{print $3 \"/\" $2 \" (\" $5 \")\"}'",
        ) {
            status.disk = Some(output.trim().to_string());
        }

        status
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
async fn exec_server_command(
    state: State<'_, AppState>,
    server_id: String,
    command: String,
) -> Result<String, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    // Get server
    let server = db
        .get_server(&server_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Server not found")?;

    let cred_id = server
        .credential_id
        .as_ref()
        .ok_or("No credential configured")?;

    // Get credential
    let credential = db
        .get_credential(cred_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Credential not found")?;

    // Create SSH connection
    let (username, port, auth_method) = match &credential.data {
        CredentialData::SshKey {
            username,
            port,
            key_path,
            passphrase,
        } => (
            username.clone(),
            *port,
            AuthMethod::Key {
                path: key_path.clone(),
                passphrase: passphrase.clone(),
            },
        ),
        CredentialData::SshAgent { username, port } => (username.clone(), *port, AuthMethod::Agent),
        _ => return Err("Credential is not SSH type".to_string()),
    };

    let ssh_conn = SshConnection {
        id: credential.id.clone(),
        name: credential.name.clone(),
        host: server.host.clone(),
        port,
        username,
        auth_method,
    };

    let mut ssh_manager = SshManager::new();
    ssh_manager.add_connection(ssh_conn);
    let conn_id = credential.id.clone();

    // Execute command
    let output =
        tokio::task::spawn_blocking(move || ssh_manager.execute_command(&conn_id, &command))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

    Ok(output)
}

// ─────────────────────────────────────────────────────────────────────────────
// Generate SSH Key
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedKeyDto {
    pub private_key_path: String,
    pub public_key_path: String,
    pub public_key_content: String,
}

#[tauri::command]
async fn generate_ssh_key(name: String) -> Result<GeneratedKeyDto, String> {
    // Get home directory
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let ssh_dir = home.join(".ssh");

    // Create .ssh directory if it doesn't exist
    std::fs::create_dir_all(&ssh_dir).map_err(|e| format!("Failed to create .ssh dir: {}", e))?;

    // Generate key name (sanitize)
    let safe_name = name
        .to_lowercase()
        .replace(' ', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>();
    let key_name = format!("id_rsa_pctrl_{}", safe_name);
    let private_key_path = ssh_dir.join(&key_name);
    let public_key_path = ssh_dir.join(format!("{}.pub", key_name));

    // Check if key already exists
    if private_key_path.exists() {
        return Err(format!("Key {} already exists", private_key_path.display()));
    }

    // Generate RSA key using ssh-keygen
    let output = std::process::Command::new("ssh-keygen")
        .args([
            "-t",
            "rsa",
            "-b",
            "4096",
            "-f",
            &private_key_path.to_string_lossy(),
            "-N",
            "", // No passphrase
            "-C",
            &format!("pctrl-{}", safe_name),
        ])
        .output()
        .map_err(|e| format!("Failed to run ssh-keygen: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ssh-keygen failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Read public key content
    let public_key_content = std::fs::read_to_string(&public_key_path)
        .map_err(|e| format!("Failed to read public key: {}", e))?;

    Ok(GeneratedKeyDto {
        private_key_path: private_key_path.to_string_lossy().to_string(),
        public_key_path: public_key_path.to_string_lossy().to_string(),
        public_key_content: public_key_content.trim().to_string(),
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Test Connection
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn test_credential_connection(
    state: State<'_, AppState>,
    credential_id: String,
    host: String,
) -> Result<String, String> {
    ensure_db(&state).await?;
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    // Get credential
    let credential = db
        .get_credential(&credential_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Credential not found")?;

    // Create SSH connection
    let (username, port, auth_method) = match &credential.data {
        CredentialData::SshKey {
            username,
            port,
            key_path,
            passphrase,
        } => (
            username.clone(),
            *port,
            AuthMethod::Key {
                path: key_path.clone(),
                passphrase: passphrase.clone(),
            },
        ),
        CredentialData::SshAgent { username, port } => (username.clone(), *port, AuthMethod::Agent),
        _ => return Err("Credential is not SSH type".to_string()),
    };

    let ssh_conn = SshConnection {
        id: credential.id.clone(),
        name: credential.name.clone(),
        host: host.clone(),
        port,
        username,
        auth_method,
    };

    let mut ssh_manager = SshManager::new();
    ssh_manager.add_connection(ssh_conn);
    let conn_id = credential.id.clone();

    // Test connection
    let result = tokio::task::spawn_blocking(move || {
        match ssh_manager.execute_command(&conn_id, "echo 'Connection OK'") {
            Ok(_) => Ok("Connection successful!".to_string()),
            Err(e) => Err(e.to_string()),
        }
    })
    .await
    .map_err(|e| e.to_string())?;

    result
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
            // Credential & SSH Commands
            list_credentials,
            add_credential,
            delete_credential,
            get_server_status,
            exec_server_command,
            test_credential_connection,
            generate_ssh_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
