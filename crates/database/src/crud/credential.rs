//! CRUD operations for credentials

use crate::Database;
use pctrl_core::{Credential, CredentialData, CredentialType, Result};

impl Database {
    /// Save a credential (insert or update)
    pub async fn save_credential(&self, credential: &Credential) -> Result<()> {
        // Serialize the credential data to JSON (will be encrypted)
        let data_json = serde_json::to_string(&credential.data)
            .map_err(|e| pctrl_core::Error::Database(format!("Failed to serialize data: {}", e)))?;

        // Encrypt the sensitive data
        let encrypted_data = self.encrypt(data_json.as_bytes())?;

        sqlx::query(
            r#"
            INSERT INTO credentials (id, name, credential_type, data, notes)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                credential_type = excluded.credential_type,
                data = excluded.data,
                notes = excluded.notes
            "#,
        )
        .bind(&credential.id)
        .bind(&credential.name)
        .bind(credential.credential_type.to_string())
        .bind(&encrypted_data)
        .bind(&credential.notes)
        .execute(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(())
    }

    /// List all credentials
    pub async fn list_credentials(&self) -> Result<Vec<Credential>> {
        let rows: Vec<(String, String, String, Vec<u8>, Option<String>)> = sqlx::query_as(
            "SELECT id, name, credential_type, data, notes FROM credentials ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        let mut credentials = Vec::new();
        for (id, name, cred_type, encrypted_data, notes) in rows {
            // Decrypt the data
            let decrypted = self.decrypt(&encrypted_data)?;
            let data_json = String::from_utf8(decrypted)
                .map_err(|e| pctrl_core::Error::Database(format!("Invalid UTF-8: {}", e)))?;

            let data: CredentialData = serde_json::from_str(&data_json)
                .map_err(|e| pctrl_core::Error::Database(format!("Failed to parse data: {}", e)))?;

            let credential_type: CredentialType = cred_type.parse().unwrap_or_default();

            credentials.push(Credential {
                id,
                name,
                credential_type,
                data,
                notes,
            });
        }

        Ok(credentials)
    }

    /// Get a credential by ID
    pub async fn get_credential(&self, id: &str) -> Result<Option<Credential>> {
        let row: Option<(String, String, String, Vec<u8>, Option<String>)> = sqlx::query_as(
            "SELECT id, name, credential_type, data, notes FROM credentials WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        match row {
            Some((id, name, cred_type, encrypted_data, notes)) => {
                let decrypted = self.decrypt(&encrypted_data)?;
                let data_json = String::from_utf8(decrypted)
                    .map_err(|e| pctrl_core::Error::Database(format!("Invalid UTF-8: {}", e)))?;

                let data: CredentialData = serde_json::from_str(&data_json).map_err(|e| {
                    pctrl_core::Error::Database(format!("Failed to parse data: {}", e))
                })?;

                let credential_type: CredentialType = cred_type.parse().unwrap_or_default();

                Ok(Some(Credential {
                    id,
                    name,
                    credential_type,
                    data,
                    notes,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get a credential by name
    pub async fn get_credential_by_name(&self, name: &str) -> Result<Option<Credential>> {
        let row: Option<(String, String, String, Vec<u8>, Option<String>)> = sqlx::query_as(
            "SELECT id, name, credential_type, data, notes FROM credentials WHERE name = ?",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        match row {
            Some((id, name, cred_type, encrypted_data, notes)) => {
                let decrypted = self.decrypt(&encrypted_data)?;
                let data_json = String::from_utf8(decrypted)
                    .map_err(|e| pctrl_core::Error::Database(format!("Invalid UTF-8: {}", e)))?;

                let data: CredentialData = serde_json::from_str(&data_json).map_err(|e| {
                    pctrl_core::Error::Database(format!("Failed to parse data: {}", e))
                })?;

                let credential_type: CredentialType = cred_type.parse().unwrap_or_default();

                Ok(Some(Credential {
                    id,
                    name,
                    credential_type,
                    data,
                    notes,
                }))
            }
            None => Ok(None),
        }
    }

    /// Remove a credential by ID
    pub async fn remove_credential(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM credentials WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Remove a credential by name
    pub async fn remove_credential_by_name(&self, name: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM credentials WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }
}
