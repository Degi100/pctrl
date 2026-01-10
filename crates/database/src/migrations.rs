//! Database schema migrations
//!
//! This module handles automatic schema migrations when the database
//! schema version is outdated.

use pctrl_core::Result;
use sqlx::sqlite::SqlitePool;

/// Current schema version
pub const CURRENT_SCHEMA_VERSION: i32 = 4;

/// Run all pending migrations
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let current_version = get_schema_version(pool).await?;

    if current_version >= CURRENT_SCHEMA_VERSION {
        return Ok(());
    }

    tracing::info!(
        "Running database migrations: v{} -> v{}",
        current_version,
        CURRENT_SCHEMA_VERSION
    );

    // Run migrations sequentially
    for version in (current_version + 1)..=CURRENT_SCHEMA_VERSION {
        run_migration(pool, version).await?;
        set_schema_version(pool, version).await?;
        tracing::info!("Migration v{} completed", version);
    }

    Ok(())
}

/// Get current schema version from metadata table
async fn get_schema_version(pool: &SqlitePool) -> Result<i32> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM metadata WHERE key = 'schema_version'")
            .fetch_optional(pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

    match row {
        Some((value,)) => value
            .parse::<i32>()
            .map_err(|e| pctrl_core::Error::Database(format!("Invalid schema version: {}", e))),
        None => Ok(1), // No version means v1 (original schema)
    }
}

/// Set schema version in metadata table
async fn set_schema_version(pool: &SqlitePool, version: i32) -> Result<()> {
    sqlx::query("INSERT OR REPLACE INTO metadata (key, value) VALUES ('schema_version', ?)")
        .bind(version.to_string())
        .execute(pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

    Ok(())
}

/// Run a specific migration
async fn run_migration(pool: &SqlitePool, version: i32) -> Result<()> {
    match version {
        2 => migrate_v2(pool).await,
        3 => migrate_v3(pool).await,
        4 => migrate_v4(pool).await,
        _ => Ok(()), // Unknown version, skip
    }
}

/// Migration v1 -> v2: Add missing columns to scripts table
async fn migrate_v2(pool: &SqlitePool) -> Result<()> {
    // Check if columns exist before adding them
    let columns = get_table_columns(pool, "scripts").await?;

    if !columns.contains(&"exit_code".to_string()) {
        sqlx::query("ALTER TABLE scripts ADD COLUMN exit_code INTEGER")
            .execute(pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;
    }

    if !columns.contains(&"last_output".to_string()) {
        sqlx::query("ALTER TABLE scripts ADD COLUMN last_output TEXT")
            .execute(pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;
    }

    Ok(())
}

/// Get list of column names for a table
async fn get_table_columns(pool: &SqlitePool, table: &str) -> Result<Vec<String>> {
    let rows: Vec<(String,)> =
        sqlx::query_as(&format!("SELECT name FROM pragma_table_info('{}')", table))
            .fetch_all(pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

    Ok(rows.into_iter().map(|(name,)| name).collect())
}

/// Migration v2 -> v3: Rename ssh_connection_id to credential_id in servers
async fn migrate_v3(pool: &SqlitePool) -> Result<()> {
    let columns = get_table_columns(pool, "servers").await?;

    // Only migrate if old column exists and new one doesn't
    if columns.contains(&"ssh_connection_id".to_string())
        && !columns.contains(&"credential_id".to_string())
    {
        // SQLite 3.25.0+ supports RENAME COLUMN
        sqlx::query("ALTER TABLE servers RENAME COLUMN ssh_connection_id TO credential_id")
            .execute(pool)
            .await
            .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;
    }

    Ok(())
}

/// Migration v3 -> v4: Fix servers FK to reference credentials instead of ssh_connections
async fn migrate_v4(pool: &SqlitePool) -> Result<()> {
    // SQLite doesn't support ALTER FK, so we need to recreate the table
    // First, clear invalid credential_id references

    // Set credential_id to NULL where it doesn't exist in credentials table
    sqlx::query(
        r#"
        UPDATE servers
        SET credential_id = NULL
        WHERE credential_id IS NOT NULL
          AND credential_id NOT IN (SELECT id FROM credentials)
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

    // Disable FK checks temporarily
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

    // Create new table with correct FK
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS servers_new (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            host TEXT NOT NULL,
            server_type TEXT DEFAULT 'vps',
            provider TEXT,
            credential_id TEXT,
            location TEXT,
            specs TEXT,
            notes TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (credential_id) REFERENCES credentials(id)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

    // Copy data from old table
    sqlx::query(
        r#"
        INSERT INTO servers_new (id, name, host, server_type, provider, credential_id, location, specs, notes, created_at)
        SELECT id, name, host, server_type, provider, credential_id, location, specs, notes, created_at
        FROM servers
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

    // Drop old table
    sqlx::query("DROP TABLE servers")
        .execute(pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

    // Rename new table
    sqlx::query("ALTER TABLE servers_new RENAME TO servers")
        .execute(pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

    // Re-enable FK checks
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await
        .map_err(|e| pctrl_core::Error::Database(e.to_string()))?;

    Ok(())
}
