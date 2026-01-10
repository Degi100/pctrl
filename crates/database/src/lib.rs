//! pctrl-database - Database layer with encryption support
//!
//! This crate provides the database abstraction for pctrl.
//! All CRUD operations are organized in the `crud` module.

#![allow(clippy::type_complexity)]

mod crud;
mod migrations;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::password_hash::SaltString;
use argon2::Argon2;
use pctrl_core::Result;
use sqlx::sqlite::SqlitePool;

/// Database manager with encryption support
pub struct Database {
    pub(crate) pool: SqlitePool,
    cipher: Option<Aes256Gcm>,
    #[allow(dead_code)]
    encryption_salt: Option<Vec<u8>>,
}

impl Database {
    /// Create a new database connection
    /// Path kann ein Dateipfad oder eine SQLite-URL sein
    pub async fn new(path: &str, password: Option<&str>) -> Result<Self> {
        // SQLite URL: mode=rwc erstellt die DB automatisch wenn sie nicht existiert
        let url = if path.starts_with("sqlite:") {
            path.to_string()
        } else {
            format!("sqlite:{}?mode=rwc", path)
        };

        let pool = SqlitePool::connect(&url)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        // Initialize metadata table first (needed for salt storage)
        Self::init_metadata_table(&pool).await?;

        let (cipher, salt) = if let Some(pwd) = password {
            // Get or create a cryptographically secure random salt
            let salt = Self::get_or_create_salt(&pool).await?;

            let key = Self::derive_key(pwd, &salt)?;
            (Some(Aes256Gcm::new(&key.into())), Some(salt.to_vec()))
        } else {
            (None, None)
        };

        let db = Self {
            pool,
            cipher,
            encryption_salt: salt,
        };
        db.init_schema().await?;

        // Run any pending migrations
        migrations::run_migrations(&db.pool).await?;

        Ok(db)
    }

    /// Initialize metadata table for storing encryption salt
    async fn init_metadata_table(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value BLOB NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Get existing salt or create a new random one
    async fn get_or_create_salt(pool: &SqlitePool) -> Result<[u8; 16]> {
        // Try to get existing salt
        let row: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT value FROM metadata WHERE key = 'encryption_salt'")
                .fetch_optional(pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        if let Some((salt_bytes,)) = row {
            // Salt exists, use it
            if salt_bytes.len() >= 16 {
                let mut salt = [0u8; 16];
                salt.copy_from_slice(&salt_bytes[..16]);
                return Ok(salt);
            }
        }

        // Generate new random salt
        use rand::RngCore;
        let mut salt = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut salt);

        // Store the salt
        sqlx::query("INSERT OR REPLACE INTO metadata (key, value) VALUES ('encryption_salt', ?)")
            .bind(salt.as_slice())
            .execute(pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(salt)
    }

    /// Initialize database schema
    async fn init_schema(&self) -> Result<()> {
        sqlx::query(SCHEMA_SQL)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Derive encryption key from password using Argon2 with a fixed salt
    fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
        use argon2::password_hash::PasswordHasher;

        // Create a SaltString from the provided salt bytes
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| pctrl_core::Error::Database(format!("Salt encoding failed: {}", e)))?;

        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| pctrl_core::Error::Database(format!("Key derivation failed: {}", e)))?;

        let hash = password_hash.hash.ok_or_else(|| {
            pctrl_core::Error::Database("Failed to get password hash".to_string())
        })?;

        let mut key = [0u8; 32];
        key.copy_from_slice(&hash.as_bytes()[..32]);
        Ok(key)
    }

    /// Encrypt data
    /// Returns nonce (12 bytes) prepended to ciphertext
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if let Some(cipher) = &self.cipher {
            use rand::RngCore;
            // Generate a cryptographically secure random nonce
            let mut nonce_bytes = [0u8; 12];
            rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let ciphertext = cipher
                .encrypt(nonce, data)
                .map_err(|e| pctrl_core::Error::Database(format!("Encryption failed: {}", e)))?;

            // Prepend nonce to ciphertext for storage
            let mut result = nonce_bytes.to_vec();
            result.extend_from_slice(&ciphertext);
            Ok(result)
        } else {
            Ok(data.to_vec())
        }
    }

    /// Decrypt data
    /// Expects nonce (12 bytes) prepended to ciphertext
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if let Some(cipher) = &self.cipher {
            if data.len() < 12 {
                return Err(pctrl_core::Error::Database(
                    "Invalid encrypted data: too short".to_string(),
                ));
            }

            // Extract nonce from the first 12 bytes
            let nonce = Nonce::from_slice(&data[..12]);
            let ciphertext = &data[12..];

            cipher
                .decrypt(nonce, ciphertext)
                .map_err(|e| pctrl_core::Error::Database(format!("Decryption failed: {}", e)))
        } else {
            Ok(data.to_vec())
        }
    }
}

/// Database schema SQL
const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS ssh_connections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    username TEXT NOT NULL,
    auth_method TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS docker_hosts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS coolify_instances (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    api_key TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS git_repos (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    remote_url TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS changelog (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    version TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS roadmap (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    priority TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- v6: PROJECTS (Core Entity)
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    stack TEXT,
    status TEXT DEFAULT 'dev',
    color TEXT,
    icon TEXT,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- v6: SERVERS
CREATE TABLE IF NOT EXISTS servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    server_type TEXT DEFAULT 'vps',
    provider TEXT,
    ssh_connection_id TEXT,
    location TEXT,
    specs TEXT,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (ssh_connection_id) REFERENCES ssh_connections(id)
);

-- v6: DOMAINS
CREATE TABLE IF NOT EXISTS domains (
    id TEXT PRIMARY KEY,
    domain TEXT NOT NULL UNIQUE,
    domain_type TEXT DEFAULT 'production',
    ssl INTEGER DEFAULT 1,
    ssl_expiry DATETIME,
    cloudflare_zone_id TEXT,
    cloudflare_record_id TEXT,
    server_id TEXT,
    container_id TEXT,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (server_id) REFERENCES servers(id)
);

-- v6: DATABASES (Credentials)
CREATE TABLE IF NOT EXISTS databases (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    db_type TEXT NOT NULL,
    host TEXT,
    port INTEGER,
    database_name TEXT,
    username TEXT,
    password TEXT,
    connection_string TEXT,
    server_id TEXT,
    container_id TEXT,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (server_id) REFERENCES servers(id)
);

-- v6: CONTAINERS
CREATE TABLE IF NOT EXISTS containers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    image TEXT,
    server_id TEXT NOT NULL,
    project_id TEXT,
    status TEXT,
    ports TEXT,
    env_vars TEXT,
    labels TEXT,
    created_at DATETIME,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (server_id) REFERENCES servers(id),
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- v6: SCRIPTS
CREATE TABLE IF NOT EXISTS scripts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    command TEXT NOT NULL,
    script_type TEXT DEFAULT 'ssh',
    server_id TEXT,
    project_id TEXT,
    docker_host_id TEXT,
    container_id TEXT,
    dangerous INTEGER DEFAULT 0,
    last_run DATETIME,
    last_result TEXT,
    exit_code INTEGER,
    last_output TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (server_id) REFERENCES servers(id),
    FOREIGN KEY (project_id) REFERENCES projects(id),
    FOREIGN KEY (docker_host_id) REFERENCES docker_hosts(id)
);

-- v6: PROJECT_RESOURCES
CREATE TABLE IF NOT EXISTS project_resources (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    role TEXT,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- v6: DISCOVERY_CACHE
CREATE TABLE IF NOT EXISTS discovery_cache (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    data_type TEXT NOT NULL,
    data TEXT NOT NULL,
    fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME,
    FOREIGN KEY (server_id) REFERENCES servers(id)
);
"#;
