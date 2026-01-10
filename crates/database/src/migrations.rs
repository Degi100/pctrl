//! Database schema migrations
//!
//! This module handles automatic schema migrations when the database
//! schema version is outdated.

use pctrl_core::Result;
use sqlx::sqlite::SqlitePool;

/// Current schema version
pub const CURRENT_SCHEMA_VERSION: i32 = 2;

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
