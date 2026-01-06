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
    pub async fn new(path: &str, password: Option<&str>) -> Result<Self> {
        let url = format!("sqlite:{}", path);
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

        Ok(config)
    }
}
