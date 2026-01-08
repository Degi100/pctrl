//! Domain CRUD operations

use crate::Database;
use pctrl_core::Result;

impl Database {
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
        let row: Option<(
            String,
            String,
            String,
            bool,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, domain, domain_type, ssl, ssl_expiry, cloudflare_zone_id, cloudflare_record_id, server_id, container_id, notes FROM domains WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(Self::row_to_domain))
    }

    /// Get a domain by domain name
    pub async fn get_domain_by_name(
        &self,
        domain_name: &str,
    ) -> Result<Option<pctrl_core::Domain>> {
        let row: Option<(
            String,
            String,
            String,
            bool,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, domain, domain_type, ssl, ssl_expiry, cloudflare_zone_id, cloudflare_record_id, server_id, container_id, notes FROM domains WHERE LOWER(domain) = LOWER(?)",
        )
        .bind(domain_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(row.map(Self::row_to_domain))
    }

    /// List all domains
    pub async fn list_domains(&self) -> Result<Vec<pctrl_core::Domain>> {
        let rows: Vec<(
            String,
            String,
            String,
            bool,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT id, domain, domain_type, ssl, ssl_expiry, cloudflare_zone_id, cloudflare_record_id, server_id, container_id, notes FROM domains ORDER BY domain",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

        Ok(rows.into_iter().map(Self::row_to_domain).collect())
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

    /// Helper to convert a row tuple to Domain
    fn row_to_domain(
        row: (
            String,
            String,
            String,
            bool,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    ) -> pctrl_core::Domain {
        let (
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
        ) = row;
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
    }
}
