use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::password_hash::SaltString;
use argon2::Argon2;
use pctrl_core::{Config, Result};
use sqlx::{sqlite::SqlitePool, Row};

/// Database manager with encryption support
pub struct Database {
    pool: SqlitePool,
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

        let (cipher, salt) = if let Some(pwd) = password {
            // TODO: In production, the salt should be randomly generated during
            // database creation and stored in a metadata table, then retrieved on
            // subsequent opens. For now, we use a deterministic salt for simplicity.
            // This allows the same password to consistently decrypt the database.
            let salt_string = format!("pctrl-salt-{}", path);
            let salt_bytes = salt_string.as_bytes();
            let mut salt = [0u8; 16];
            let copy_len = 16.min(salt_bytes.len());
            salt[..copy_len].copy_from_slice(&salt_bytes[..copy_len]);

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
        Ok(db)
    }

    /// Initialize database schema
    async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
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

            -- ═══════════════════════════════════════════════════════════════
            -- v6: PROJECTS (Core Entity)
            -- ═══════════════════════════════════════════════════════════════

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

            -- ═══════════════════════════════════════════════════════════════
            -- v6: SERVERS (eigenständig, nicht nur SSH)
            -- ═══════════════════════════════════════════════════════════════

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

            -- ═══════════════════════════════════════════════════════════════
            -- v6: DOMAINS
            -- ═══════════════════════════════════════════════════════════════

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

            -- ═══════════════════════════════════════════════════════════════
            -- v6: DATABASES (Credentials encrypted!)
            -- ═══════════════════════════════════════════════════════════════

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

            -- ═══════════════════════════════════════════════════════════════
            -- v6: CONTAINERS (erweitert)
            -- ═══════════════════════════════════════════════════════════════

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

            -- ═══════════════════════════════════════════════════════════════
            -- v6: SCRIPTS (Custom Commands)
            -- ═══════════════════════════════════════════════════════════════

            CREATE TABLE IF NOT EXISTS scripts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                command TEXT NOT NULL,
                script_type TEXT DEFAULT 'ssh',
                server_id TEXT,
                project_id TEXT,
                dangerous INTEGER DEFAULT 0,
                last_run DATETIME,
                last_result TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (server_id) REFERENCES servers(id),
                FOREIGN KEY (project_id) REFERENCES projects(id)
            );

            -- ═══════════════════════════════════════════════════════════════
            -- v6: PROJECT_RESOURCES (Verknüpfungstabelle)
            -- ═══════════════════════════════════════════════════════════════

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

            -- ═══════════════════════════════════════════════════════════════
            -- v6: DISCOVERY_CACHE (für schnelle Refreshes)
            -- ═══════════════════════════════════════════════════════════════

            CREATE TABLE IF NOT EXISTS discovery_cache (
                id TEXT PRIMARY KEY,
                server_id TEXT NOT NULL,
                data_type TEXT NOT NULL,
                data TEXT NOT NULL,
                fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                expires_at DATETIME,
                FOREIGN KEY (server_id) REFERENCES servers(id)
            );
            "#,
        )
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

    /// Save configuration to database
    pub async fn save_config(&self, config: &Config) -> Result<()> {
        // Save SSH connections
        for conn in &config.ssh_connections {
            let auth_method = serde_json::to_string(&conn.auth_method)
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

            sqlx::query(
                "INSERT OR REPLACE INTO ssh_connections (id, name, host, port, username, auth_method) 
                 VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(&conn.id)
            .bind(&conn.name)
            .bind(&conn.host)
            .bind(conn.port as i64)
            .bind(&conn.username)
            .bind(&auth_method)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;
        }

        Ok(())
    }

    /// Load configuration from database
    pub async fn load_config(&self) -> Result<Config> {
        let mut config = Config::default();

        // Load SSH connections
        let rows =
            sqlx::query("SELECT id, name, host, port, username, auth_method FROM ssh_connections")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        for row in rows {
            let auth_method: String = row.get("auth_method");
            let auth_method = serde_json::from_str(&auth_method)
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

            config.ssh_connections.push(pctrl_core::SshConnection {
                id: row.get("id"),
                name: row.get("name"),
                host: row.get("host"),
                port: row.get::<i64, _>("port") as u16,
                username: row.get("username"),
                auth_method,
            });
        }

        // Load Docker hosts
        let rows = sqlx::query("SELECT id, name, url FROM docker_hosts")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        for row in rows {
            config.docker_hosts.push(pctrl_core::DockerHost {
                id: row.get("id"),
                name: row.get("name"),
                url: row.get("url"),
            });
        }

        // Load Coolify instances
        let rows = sqlx::query("SELECT id, name, url, api_key FROM coolify_instances")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        for row in rows {
            config.coolify_instances.push(pctrl_core::CoolifyInstance {
                id: row.get("id"),
                name: row.get("name"),
                url: row.get("url"),
                api_key: row.get("api_key"),
            });
        }

        // Load Git repositories
        let rows = sqlx::query("SELECT id, name, path, remote_url FROM git_repos")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        for row in rows {
            config.git_repos.push(pctrl_core::GitRepo {
                id: row.get("id"),
                name: row.get("name"),
                path: row.get("path"),
                remote_url: row.get("remote_url"),
            });
        }

        Ok(config)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SSH Connection Methods
    // ─────────────────────────────────────────────────────────────────────────

    /// Add or update a single SSH connection
    pub async fn save_ssh_connection(&self, conn: &pctrl_core::SshConnection) -> Result<()> {
        let auth_method = serde_json::to_string(&conn.auth_method)
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        sqlx::query(
            "INSERT OR REPLACE INTO ssh_connections (id, name, host, port, username, auth_method)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&conn.id)
        .bind(&conn.name)
        .bind(&conn.host)
        .bind(conn.port as i64)
        .bind(&conn.username)
        .bind(&auth_method)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Remove an SSH connection by ID
    pub async fn remove_ssh_connection(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM ssh_connections WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if an SSH connection exists
    pub async fn ssh_connection_exists(&self, id: &str) -> Result<bool> {
        let row: Option<(i64,)> =
            sqlx::query_as("SELECT COUNT(*) FROM ssh_connections WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(|(count,)| count > 0).unwrap_or(false))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Docker Host Methods
    // ─────────────────────────────────────────────────────────────────────────

    /// Add or update a Docker host
    pub async fn save_docker_host(&self, host: &pctrl_core::DockerHost) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO docker_hosts (id, name, url)
             VALUES (?, ?, ?)",
        )
        .bind(&host.id)
        .bind(&host.name)
        .bind(&host.url)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Remove a Docker host by ID
    pub async fn remove_docker_host(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM docker_hosts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if a Docker host exists
    pub async fn docker_host_exists(&self, id: &str) -> Result<bool> {
        let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM docker_hosts WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(|(count,)| count > 0).unwrap_or(false))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Coolify Instance Methods
    // ─────────────────────────────────────────────────────────────────────────

    /// Add or update a Coolify instance
    pub async fn save_coolify_instance(
        &self,
        instance: &pctrl_core::CoolifyInstance,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO coolify_instances (id, name, url, api_key)
             VALUES (?, ?, ?, ?)",
        )
        .bind(&instance.id)
        .bind(&instance.name)
        .bind(&instance.url)
        .bind(&instance.api_key)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Remove a Coolify instance by ID
    pub async fn remove_coolify_instance(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM coolify_instances WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if a Coolify instance exists
    pub async fn coolify_instance_exists(&self, id: &str) -> Result<bool> {
        let row: Option<(i64,)> =
            sqlx::query_as("SELECT COUNT(*) FROM coolify_instances WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(|(count,)| count > 0).unwrap_or(false))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Git Repository Methods
    // ─────────────────────────────────────────────────────────────────────────

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

    // ═══════════════════════════════════════════════════════════════════════════
    // v6: PROJECT METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Save a project
    pub async fn save_project(&self, project: &pctrl_core::Project) -> Result<()> {
        let stack = serde_json::to_string(&project.stack)
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        sqlx::query(
            "INSERT OR REPLACE INTO projects (id, name, description, stack, status, color, icon, notes, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
        )
        .bind(&project.id)
        .bind(&project.name)
        .bind(&project.description)
        .bind(&stack)
        .bind(project.status.to_string())
        .bind(&project.color)
        .bind(&project.icon)
        .bind(&project.notes)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Get a project by ID
    pub async fn get_project(&self, id: &str) -> Result<Option<pctrl_core::Project>> {
        let row: Option<(String, String, Option<String>, Option<String>, String, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, name, description, stack, status, color, icon, notes FROM projects WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        if let Some((id, name, description, stack, status, color, icon, notes)) = row {
            let stack: Vec<String> = stack
                .map(|s| serde_json::from_str(&s).unwrap_or_default())
                .unwrap_or_default();
            let status = status.parse().unwrap_or_default();

            Ok(Some(pctrl_core::Project {
                id,
                name,
                description,
                stack,
                status,
                color,
                icon,
                notes,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get a project by name (case-insensitive)
    pub async fn get_project_by_name(&self, name: &str) -> Result<Option<pctrl_core::Project>> {
        let row: Option<(String, String, Option<String>, Option<String>, String, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, name, description, stack, status, color, icon, notes FROM projects WHERE LOWER(name) = LOWER(?)")
                .bind(name)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        if let Some((id, name, description, stack, status, color, icon, notes)) = row {
            let stack: Vec<String> = stack
                .map(|s| serde_json::from_str(&s).unwrap_or_default())
                .unwrap_or_default();
            let status = status.parse().unwrap_or_default();

            Ok(Some(pctrl_core::Project {
                id,
                name,
                description,
                stack,
                status,
                color,
                icon,
                notes,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all projects
    pub async fn list_projects(&self) -> Result<Vec<pctrl_core::Project>> {
        let rows: Vec<(String, String, Option<String>, Option<String>, String, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, name, description, stack, status, color, icon, notes FROM projects ORDER BY name")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let projects = rows
            .into_iter()
            .map(|(id, name, description, stack, status, color, icon, notes)| {
                let stack: Vec<String> = stack
                    .map(|s| serde_json::from_str(&s).unwrap_or_default())
                    .unwrap_or_default();
                let status = status.parse().unwrap_or_default();

                pctrl_core::Project {
                    id,
                    name,
                    description,
                    stack,
                    status,
                    color,
                    icon,
                    notes,
                }
            })
            .collect();

        Ok(projects)
    }

    /// Remove a project by ID
    pub async fn remove_project(&self, id: &str) -> Result<bool> {
        // Also remove all project_resources for this project
        sqlx::query("DELETE FROM project_resources WHERE project_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let result = sqlx::query("DELETE FROM projects WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if a project exists
    pub async fn project_exists(&self, id: &str) -> Result<bool> {
        let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM projects WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(|(count,)| count > 0).unwrap_or(false))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // v6: SERVER METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Save a server
    pub async fn save_server(&self, server: &pctrl_core::Server) -> Result<()> {
        let specs = server.specs.as_ref()
            .map(|s| serde_json::to_string(s).unwrap_or_default());

        sqlx::query(
            "INSERT OR REPLACE INTO servers (id, name, host, server_type, provider, ssh_connection_id, location, specs, notes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&server.id)
        .bind(&server.name)
        .bind(&server.host)
        .bind(server.server_type.to_string())
        .bind(&server.provider)
        .bind(&server.ssh_connection_id)
        .bind(&server.location)
        .bind(&specs)
        .bind(&server.notes)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Get a server by ID
    pub async fn get_server(&self, id: &str) -> Result<Option<pctrl_core::Server>> {
        let row: Option<(String, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, name, host, server_type, provider, ssh_connection_id, location, specs, notes FROM servers WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        if let Some((id, name, host, server_type, provider, ssh_connection_id, location, specs, notes)) = row {
            let server_type = server_type.parse().unwrap_or_default();
            let specs = specs.and_then(|s| serde_json::from_str(&s).ok());

            Ok(Some(pctrl_core::Server {
                id,
                name,
                host,
                server_type,
                provider,
                ssh_connection_id,
                location,
                specs,
                notes,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get a server by name (case-insensitive)
    pub async fn get_server_by_name(&self, name: &str) -> Result<Option<pctrl_core::Server>> {
        let row: Option<(String, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, name, host, server_type, provider, ssh_connection_id, location, specs, notes FROM servers WHERE LOWER(name) = LOWER(?)")
                .bind(name)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        if let Some((id, name, host, server_type, provider, ssh_connection_id, location, specs, notes)) = row {
            let server_type = server_type.parse().unwrap_or_default();
            let specs = specs.and_then(|s| serde_json::from_str(&s).ok());

            Ok(Some(pctrl_core::Server {
                id,
                name,
                host,
                server_type,
                provider,
                ssh_connection_id,
                location,
                specs,
                notes,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all servers
    pub async fn list_servers(&self) -> Result<Vec<pctrl_core::Server>> {
        let rows: Vec<(String, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, name, host, server_type, provider, ssh_connection_id, location, specs, notes FROM servers ORDER BY name")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let servers = rows
            .into_iter()
            .map(|(id, name, host, server_type, provider, ssh_connection_id, location, specs, notes)| {
                let server_type = server_type.parse().unwrap_or_default();
                let specs = specs.and_then(|s| serde_json::from_str(&s).ok());

                pctrl_core::Server {
                    id,
                    name,
                    host,
                    server_type,
                    provider,
                    ssh_connection_id,
                    location,
                    specs,
                    notes,
                }
            })
            .collect();

        Ok(servers)
    }

    /// Remove a server by ID
    pub async fn remove_server(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM servers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if a server exists
    pub async fn server_exists(&self, id: &str) -> Result<bool> {
        let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM servers WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(|(count,)| count > 0).unwrap_or(false))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // v6: DOMAIN METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Save a domain
    pub async fn save_domain(&self, domain: &pctrl_core::Domain) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO domains (id, domain, domain_type, ssl, ssl_expiry, cloudflare_zone_id, cloudflare_record_id, server_id, container_id, notes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&domain.id)
        .bind(&domain.domain)
        .bind(domain.domain_type.to_string())
        .bind(domain.ssl)
        .bind(&domain.ssl_expiry)
        .bind(&domain.cloudflare_zone_id)
        .bind(&domain.cloudflare_record_id)
        .bind(&domain.server_id)
        .bind(&domain.container_id)
        .bind(&domain.notes)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Get a domain by ID
    pub async fn get_domain(&self, id: &str) -> Result<Option<pctrl_core::Domain>> {
        let row: Option<(String, String, String, bool, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, domain, domain_type, ssl, ssl_expiry, cloudflare_zone_id, cloudflare_record_id, server_id, container_id, notes FROM domains WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        if let Some((id, domain, domain_type, ssl, ssl_expiry, cloudflare_zone_id, cloudflare_record_id, server_id, container_id, notes)) = row {
            let domain_type = domain_type.parse().unwrap_or_default();

            Ok(Some(pctrl_core::Domain {
                id,
                domain,
                domain_type,
                ssl,
                ssl_expiry,
                cloudflare_zone_id,
                cloudflare_record_id,
                server_id,
                container_id,
                notes,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get a domain by domain name
    pub async fn get_domain_by_name(&self, domain_name: &str) -> Result<Option<pctrl_core::Domain>> {
        let row: Option<(String, String, String, bool, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, domain, domain_type, ssl, ssl_expiry, cloudflare_zone_id, cloudflare_record_id, server_id, container_id, notes FROM domains WHERE LOWER(domain) = LOWER(?)")
                .bind(domain_name)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        if let Some((id, domain, domain_type, ssl, ssl_expiry, cloudflare_zone_id, cloudflare_record_id, server_id, container_id, notes)) = row {
            let domain_type = domain_type.parse().unwrap_or_default();

            Ok(Some(pctrl_core::Domain {
                id,
                domain,
                domain_type,
                ssl,
                ssl_expiry,
                cloudflare_zone_id,
                cloudflare_record_id,
                server_id,
                container_id,
                notes,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all domains
    pub async fn list_domains(&self) -> Result<Vec<pctrl_core::Domain>> {
        let rows: Vec<(String, String, String, bool, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, domain, domain_type, ssl, ssl_expiry, cloudflare_zone_id, cloudflare_record_id, server_id, container_id, notes FROM domains ORDER BY domain")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let domains = rows
            .into_iter()
            .map(|(id, domain, domain_type, ssl, ssl_expiry, cloudflare_zone_id, cloudflare_record_id, server_id, container_id, notes)| {
                let domain_type = domain_type.parse().unwrap_or_default();

                pctrl_core::Domain {
                    id,
                    domain,
                    domain_type,
                    ssl,
                    ssl_expiry,
                    cloudflare_zone_id,
                    cloudflare_record_id,
                    server_id,
                    container_id,
                    notes,
                }
            })
            .collect();

        Ok(domains)
    }

    /// Remove a domain by ID
    pub async fn remove_domain(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM domains WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // v6: DATABASE CREDENTIALS METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Save database credentials
    pub async fn save_database_credentials(&self, db_creds: &pctrl_core::DatabaseCredentials) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO databases (id, name, db_type, host, port, database_name, username, password, connection_string, server_id, container_id, notes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&db_creds.id)
        .bind(&db_creds.name)
        .bind(db_creds.db_type.to_string())
        .bind(&db_creds.host)
        .bind(db_creds.port.map(|p| p as i64))
        .bind(&db_creds.database_name)
        .bind(&db_creds.username)
        .bind(&db_creds.password)
        .bind(&db_creds.connection_string)
        .bind(&db_creds.server_id)
        .bind(&db_creds.container_id)
        .bind(&db_creds.notes)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Get database credentials by ID
    pub async fn get_database_credentials(&self, id: &str) -> Result<Option<pctrl_core::DatabaseCredentials>> {
        let row: Option<(String, String, String, Option<String>, Option<i64>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, name, db_type, host, port, database_name, username, password, connection_string, server_id, container_id, notes FROM databases WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        if let Some((id, name, db_type, host, port, database_name, username, password, connection_string, server_id, container_id, notes)) = row {
            let db_type = db_type.parse().unwrap_or_default();

            Ok(Some(pctrl_core::DatabaseCredentials {
                id,
                name,
                db_type,
                host,
                port: port.map(|p| p as u16),
                database_name,
                username,
                password,
                connection_string,
                server_id,
                container_id,
                notes,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get database credentials by name (case-insensitive)
    pub async fn get_database_credentials_by_name(&self, name: &str) -> Result<Option<pctrl_core::DatabaseCredentials>> {
        let row: Option<(String, String, String, Option<String>, Option<i64>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, name, db_type, host, port, database_name, username, password, connection_string, server_id, container_id, notes FROM databases WHERE LOWER(name) = LOWER(?)")
                .bind(name)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        if let Some((id, name, db_type, host, port, database_name, username, password, connection_string, server_id, container_id, notes)) = row {
            let db_type = db_type.parse().unwrap_or_default();

            Ok(Some(pctrl_core::DatabaseCredentials {
                id,
                name,
                db_type,
                host,
                port: port.map(|p| p as u16),
                database_name,
                username,
                password,
                connection_string,
                server_id,
                container_id,
                notes,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all database credentials
    pub async fn list_database_credentials(&self) -> Result<Vec<pctrl_core::DatabaseCredentials>> {
        let rows: Vec<(String, String, String, Option<String>, Option<i64>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, name, db_type, host, port, database_name, username, password, connection_string, server_id, container_id, notes FROM databases ORDER BY name")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let databases = rows
            .into_iter()
            .map(|(id, name, db_type, host, port, database_name, username, password, connection_string, server_id, container_id, notes)| {
                let db_type = db_type.parse().unwrap_or_default();

                pctrl_core::DatabaseCredentials {
                    id,
                    name,
                    db_type,
                    host,
                    port: port.map(|p| p as u16),
                    database_name,
                    username,
                    password,
                    connection_string,
                    server_id,
                    container_id,
                    notes,
                }
            })
            .collect();

        Ok(databases)
    }

    /// Remove database credentials by ID
    pub async fn remove_database_credentials(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM databases WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // v6: SCRIPT METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Save a script
    pub async fn save_script(&self, script: &pctrl_core::Script) -> Result<()> {
        let last_result = script.last_result.as_ref().map(|r| r.to_string());

        sqlx::query(
            "INSERT OR REPLACE INTO scripts (id, name, description, command, script_type, server_id, project_id, dangerous, last_run, last_result)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&script.id)
        .bind(&script.name)
        .bind(&script.description)
        .bind(&script.command)
        .bind(script.script_type.to_string())
        .bind(&script.server_id)
        .bind(&script.project_id)
        .bind(script.dangerous)
        .bind(&script.last_run)
        .bind(&last_result)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Get a script by ID
    pub async fn get_script(&self, id: &str) -> Result<Option<pctrl_core::Script>> {
        let row: Option<(String, String, Option<String>, String, String, Option<String>, Option<String>, bool, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, name, description, command, script_type, server_id, project_id, dangerous, last_run, last_result FROM scripts WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        if let Some((id, name, description, command, script_type, server_id, project_id, dangerous, last_run, last_result)) = row {
            let script_type = script_type.parse().unwrap_or_default();
            let last_result = last_result.and_then(|r| match r.as_str() {
                "success" => Some(pctrl_core::ScriptResult::Success),
                "error" => Some(pctrl_core::ScriptResult::Error),
                _ => None,
            });

            Ok(Some(pctrl_core::Script {
                id,
                name,
                description,
                command,
                script_type,
                server_id,
                project_id,
                dangerous,
                last_run,
                last_result,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all scripts
    pub async fn list_scripts(&self) -> Result<Vec<pctrl_core::Script>> {
        let rows: Vec<(String, String, Option<String>, String, String, Option<String>, Option<String>, bool, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, name, description, command, script_type, server_id, project_id, dangerous, last_run, last_result FROM scripts ORDER BY name")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let scripts = rows
            .into_iter()
            .map(|(id, name, description, command, script_type, server_id, project_id, dangerous, last_run, last_result)| {
                let script_type = script_type.parse().unwrap_or_default();
                let last_result = last_result.and_then(|r| match r.as_str() {
                    "success" => Some(pctrl_core::ScriptResult::Success),
                    "error" => Some(pctrl_core::ScriptResult::Error),
                    _ => None,
                });

                pctrl_core::Script {
                    id,
                    name,
                    description,
                    command,
                    script_type,
                    server_id,
                    project_id,
                    dangerous,
                    last_run,
                    last_result,
                }
            })
            .collect();

        Ok(scripts)
    }

    /// List scripts for a project
    pub async fn list_scripts_for_project(&self, project_id: &str) -> Result<Vec<pctrl_core::Script>> {
        let rows: Vec<(String, String, Option<String>, String, String, Option<String>, Option<String>, bool, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, name, description, command, script_type, server_id, project_id, dangerous, last_run, last_result FROM scripts WHERE project_id = ? ORDER BY name")
                .bind(project_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let scripts = rows
            .into_iter()
            .map(|(id, name, description, command, script_type, server_id, project_id, dangerous, last_run, last_result)| {
                let script_type = script_type.parse().unwrap_or_default();
                let last_result = last_result.and_then(|r| match r.as_str() {
                    "success" => Some(pctrl_core::ScriptResult::Success),
                    "error" => Some(pctrl_core::ScriptResult::Error),
                    _ => None,
                });

                pctrl_core::Script {
                    id,
                    name,
                    description,
                    command,
                    script_type,
                    server_id,
                    project_id,
                    dangerous,
                    last_run,
                    last_result,
                }
            })
            .collect();

        Ok(scripts)
    }

    /// Remove a script by ID
    pub async fn remove_script(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM scripts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // v6: PROJECT RESOURCE METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Link a resource to a project
    pub async fn link_project_resource(&self, resource: &pctrl_core::ProjectResource) -> Result<()> {
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
    pub async fn get_project_resources(&self, project_id: &str) -> Result<Vec<pctrl_core::ProjectResource>> {
        let rows: Vec<(String, String, String, String, Option<String>, Option<String>)> =
            sqlx::query_as("SELECT id, project_id, resource_type, resource_id, role, notes FROM project_resources WHERE project_id = ?")
                .bind(project_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let resources = rows
            .into_iter()
            .map(|(id, project_id, resource_type, resource_id, role, notes)| {
                let resource_type = resource_type.parse().unwrap_or(pctrl_core::ResourceType::Server);

                pctrl_core::ProjectResource {
                    id,
                    project_id,
                    resource_type,
                    resource_id,
                    role,
                    notes,
                }
            })
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
    pub async fn get_projects_for_resource(&self, resource_type: &pctrl_core::ResourceType, resource_id: &str) -> Result<Vec<String>> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT project_id FROM project_resources WHERE resource_type = ? AND resource_id = ?")
                .bind(resource_type.to_string())
                .bind(resource_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }
}
