//! Database credentials command handler

use crate::DatabaseCommands;
use pctrl_core::{DatabaseCredentials, DatabaseType};
use pctrl_database::Database;

pub async fn handle(command: DatabaseCommands, db: &Database) -> anyhow::Result<()> {
    match command {
        DatabaseCommands::List => {
            let databases = db.list_database_credentials().await?;
            if databases.is_empty() {
                println!("No database credentials configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl database add <name> -t <type> -u <user> -P <password>");
            } else {
                println!("Databases ({}):", databases.len());
                println!();
                for creds in databases {
                    let host_str = creds
                        .host
                        .clone()
                        .unwrap_or_else(|| "localhost".to_string());
                    println!("  🗄️  {} [{}] - {}", creds.name, creds.db_type, host_str);
                }
            }
        }

        DatabaseCommands::Add {
            name,
            db_type,
            host,
            port,
            database,
            user,
            password,
            connection_string,
            server,
            container,
        } => {
            let id = name.to_lowercase().replace(' ', "-");

            if db.get_database_credentials_by_name(&name).await?.is_some() {
                anyhow::bail!("Database '{}' already exists.", name);
            }

            let db_type: DatabaseType = db_type.parse().map_err(|e: String| anyhow::anyhow!(e))?;

            let creds = DatabaseCredentials {
                id: id.clone(),
                name: name.clone(),
                db_type: db_type.clone(),
                host,
                port,
                database_name: database,
                username: user,
                password,
                connection_string,
                server_id: server.clone(),
                container_id: container.clone(),
                notes: None,
            };

            db.save_database_credentials(&creds).await?;

            println!("✓ Database credentials added:");
            println!();
            println!("  Name: {}", name);
            println!("  ID:   {}", id);
            println!("  Type: {}", db_type);
            if let Some(s) = &server {
                println!("  Server: {}", s);
            }
            if let Some(c) = &container {
                println!("  Container: {}", c);
            }
        }

        DatabaseCommands::Show { name } => {
            let creds = db
                .get_database_credentials_by_name(&name)
                .await?
                .or(db.get_database_credentials(&name).await?)
                .ok_or_else(|| anyhow::anyhow!("Database '{}' not found", name))?;

            println!();
            println!("  🗄️  {}", creds.name);
            println!("  ─────────────────────────────");
            println!("  ID:       {}", creds.id);
            println!("  Type:     {}", creds.db_type);
            if let Some(h) = &creds.host {
                println!("  Host:     {}", h);
            }
            if let Some(p) = creds.port {
                println!("  Port:     {}", p);
            }
            if let Some(d) = &creds.database_name {
                println!("  Database: {}", d);
            }
            if let Some(u) = &creds.username {
                println!("  User:     {}", u);
            }
            if creds.password.is_some() {
                println!("  Password: ********");
            }
            if let Some(s) = &creds.server_id {
                println!("  Server:   {}", s);
            }
            if let Some(c) = &creds.container_id {
                println!("  Container: {}", c);
            }
            println!();
        }

        DatabaseCommands::Get { name, field } => {
            let creds = db
                .get_database_credentials_by_name(&name)
                .await?
                .or(db.get_database_credentials(&name).await?)
                .ok_or_else(|| anyhow::anyhow!("Database '{}' not found", name))?;

            let value = match field.to_lowercase().as_str() {
                "user" | "username" => creds.username.clone(),
                "pass" | "password" => creds.password.clone(),
                "host" => creds.host.clone(),
                "port" => creds.port.map(|p| p.to_string()),
                "database" | "db" => creds.database_name.clone(),
                "url" | "connection_string" => creds.connection_string.clone(),
                _ => anyhow::bail!(
                    "Unknown field: {}. Use: user, pass, host, port, database, url",
                    field
                ),
            };

            if let Some(v) = value {
                println!("{}", v);
            } else {
                anyhow::bail!("Field '{}' is not set for database '{}'", field, name);
            }
        }

        DatabaseCommands::Remove { name } => {
            let creds = db
                .get_database_credentials_by_name(&name)
                .await?
                .or(db.get_database_credentials(&name).await?)
                .ok_or_else(|| anyhow::anyhow!("Database '{}' not found", name))?;

            if db.remove_database_credentials(&creds.id).await? {
                println!("✓ Database '{}' removed", creds.name);
            }
        }
    }

    Ok(())
}
