//! Credential command handlers

use crate::style;
use pctrl_core::{Credential, CredentialData, CredentialType};
use pctrl_database::Database;
use uuid::Uuid;

/// Handle credential list command
pub async fn handle_list(db: &Database) -> anyhow::Result<()> {
    let credentials = db.list_credentials().await?;

    if credentials.is_empty() {
        println!("{}", style::dim("No credentials found."));
        println!(
            "{}",
            style::dim("Add one with: pctrl credential add <name> --type ssh --user root --key ~/.ssh/id_rsa")
        );
        return Ok(());
    }

    println!("{}", style::header("Credentials"));
    println!();

    for cred in credentials {
        let type_badge = match cred.credential_type {
            CredentialType::SshKey => style::info_text("[SSH]"),
            CredentialType::SshAgent => style::info_text("[AGENT]"),
            CredentialType::ApiToken => style::warning_text("[API]"),
            CredentialType::BasicAuth => style::dim("[BASIC]"),
            CredentialType::OAuth => style::success_text("[OAUTH]"),
        };

        let details = match &cred.data {
            CredentialData::SshKey {
                username,
                port,
                key_path,
                ..
            } => {
                format!("{}@:{} ({})", username, port, key_path)
            }
            CredentialData::SshAgent { username, port } => {
                format!("{}@:{} (agent)", username, port)
            }
            CredentialData::ApiToken { url, .. } => url.as_deref().unwrap_or("no url").to_string(),
            CredentialData::BasicAuth { username, url, .. } => {
                format!("{} @ {}", username, url.as_deref().unwrap_or("no url"))
            }
            CredentialData::OAuth { url, .. } => url.as_deref().unwrap_or("no url").to_string(),
        };

        println!(
            "  {} {} {}",
            type_badge,
            style::bold(&cred.name),
            style::dim(&details)
        );
    }

    println!();
    Ok(())
}

/// Handle credential add command
pub async fn handle_add(
    db: &Database,
    name: String,
    cred_type: String,
    user: Option<String>,
    port: Option<u16>,
    key: Option<String>,
    token: Option<String>,
    password: Option<String>,
    url: Option<String>,
) -> anyhow::Result<()> {
    let credential_type: CredentialType =
        cred_type.parse().map_err(|e: String| anyhow::anyhow!(e))?;

    let data = match credential_type {
        CredentialType::SshKey => {
            let username = user.ok_or_else(|| anyhow::anyhow!("SSH credentials require --user"))?;
            let key_path = key.ok_or_else(|| anyhow::anyhow!("SSH credentials require --key"))?;

            // Expand ~ to home directory
            let expanded_key_path = if key_path.starts_with("~/") {
                if let Some(home) = dirs::home_dir() {
                    home.join(&key_path[2..]).to_string_lossy().to_string()
                } else {
                    key_path
                }
            } else {
                key_path
            };

            CredentialData::SshKey {
                username,
                port: port.unwrap_or(22),
                key_path: expanded_key_path,
                passphrase: password,
            }
        }
        CredentialType::SshAgent => {
            let username =
                user.ok_or_else(|| anyhow::anyhow!("SSH Agent credentials require --user"))?;
            CredentialData::SshAgent {
                username,
                port: port.unwrap_or(22),
            }
        }
        CredentialType::ApiToken => {
            let token_val =
                token.ok_or_else(|| anyhow::anyhow!("API token credentials require --token"))?;
            CredentialData::ApiToken {
                token: token_val,
                url,
            }
        }
        CredentialType::BasicAuth => {
            let username = user.ok_or_else(|| anyhow::anyhow!("Basic auth requires --user"))?;
            let pass = password.ok_or_else(|| anyhow::anyhow!("Basic auth requires --password"))?;
            CredentialData::BasicAuth {
                username,
                password: pass,
                url,
            }
        }
        CredentialType::OAuth => {
            let token_val = token.ok_or_else(|| anyhow::anyhow!("OAuth requires --token"))?;
            CredentialData::OAuth {
                access_token: token_val,
                refresh_token: None,
                expires_at: None,
                url,
            }
        }
    };

    let credential = Credential {
        id: Uuid::new_v4().to_string(),
        name: name.clone(),
        credential_type,
        data,
        notes: None,
    };

    db.save_credential(&credential).await?;
    println!("{} Credential '{}' added.", style::success_text("✓"), name);

    Ok(())
}

/// Handle credential show command
pub async fn handle_show(db: &Database, name: String) -> anyhow::Result<()> {
    let credential = db
        .get_credential_by_name(&name)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Credential '{}' not found", name))?;

    println!(
        "{}",
        style::header(&format!("Credential: {}", credential.name))
    );
    println!();
    println!("  {} {}", style::dim("ID:"), credential.id);
    println!("  {} {}", style::dim("Type:"), credential.credential_type);

    match &credential.data {
        CredentialData::SshKey {
            username,
            port,
            key_path,
            passphrase,
        } => {
            println!("  {} {}", style::dim("Username:"), username);
            println!("  {} {}", style::dim("Port:"), port);
            println!("  {} {}", style::dim("Key Path:"), key_path);
            println!(
                "  {} {}",
                style::dim("Passphrase:"),
                if passphrase.is_some() {
                    "***"
                } else {
                    "(none)"
                }
            );
        }
        CredentialData::SshAgent { username, port } => {
            println!("  {} {}", style::dim("Username:"), username);
            println!("  {} {}", style::dim("Port:"), port);
            println!("  {} {}", style::dim("Auth:"), "SSH Agent");
        }
        CredentialData::ApiToken { token, url } => {
            println!(
                "  {} {}***",
                style::dim("Token:"),
                &token[..token.len().min(8)]
            );
            if let Some(u) = url {
                println!("  {} {}", style::dim("URL:"), u);
            }
        }
        CredentialData::BasicAuth { username, url, .. } => {
            println!("  {} {}", style::dim("Username:"), username);
            println!("  {} ***", style::dim("Password:"));
            if let Some(u) = url {
                println!("  {} {}", style::dim("URL:"), u);
            }
        }
        CredentialData::OAuth {
            url, expires_at, ..
        } => {
            println!("  {} ***", style::dim("Token:"));
            if let Some(u) = url {
                println!("  {} {}", style::dim("URL:"), u);
            }
            if let Some(exp) = expires_at {
                println!("  {} {}", style::dim("Expires:"), exp);
            }
        }
    }

    if let Some(notes) = &credential.notes {
        println!("  {} {}", style::dim("Notes:"), notes);
    }

    println!();
    Ok(())
}

/// Handle credential remove command
pub async fn handle_remove(db: &Database, name: String) -> anyhow::Result<()> {
    let removed = db.remove_credential_by_name(&name).await?;

    if removed {
        println!(
            "{} Credential '{}' removed.",
            style::success_text("✓"),
            name
        );
    } else {
        println!(
            "{} Credential '{}' not found.",
            style::error_text("✗"),
            name
        );
    }

    Ok(())
}
