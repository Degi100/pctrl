//! Domain command handler

use crate::DomainCommands;
use pctrl_core::{Domain, DomainType};
use pctrl_database::Database;

pub async fn handle(command: DomainCommands, db: &Database) -> anyhow::Result<()> {
    match command {
        DomainCommands::List => {
            let domains = db.list_domains().await?;
            if domains.is_empty() {
                println!("No domains configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl domain add <domain> [-t type] [-s server]");
            } else {
                println!("Domains ({}):", domains.len());
                println!();
                for domain in domains {
                    let ssl_icon = if domain.ssl { "🔒" } else { "🔓" };
                    println!("  {} {} [{}]", ssl_icon, domain.domain, domain.domain_type);
                }
            }
        }

        DomainCommands::Add {
            domain,
            domain_type,
            server,
            ssl,
            ssl_expiry,
            cloudflare_zone,
            cloudflare_record,
        } => {
            let id = domain.replace('.', "-").to_lowercase();

            if db.get_domain_by_name(&domain).await?.is_some() {
                anyhow::bail!("Domain '{}' already exists.", domain);
            }

            let domain_type: DomainType = domain_type.parse().unwrap_or_default();

            let dom = Domain {
                id: id.clone(),
                domain: domain.clone(),
                domain_type: domain_type.clone(),
                ssl,
                ssl_expiry: ssl_expiry.clone(),
                cloudflare_zone_id: cloudflare_zone.clone(),
                cloudflare_record_id: cloudflare_record.clone(),
                server_id: server.clone(),
                container_id: None,
                notes: None,
            };

            db.save_domain(&dom).await?;

            println!("✓ Domain added:");
            println!();
            println!("  Domain: {}", domain);
            println!("  ID:     {}", id);
            println!("  Type:   {}", domain_type);
            println!("  SSL:    {}", if ssl { "enabled" } else { "disabled" });
            if let Some(exp) = &ssl_expiry {
                println!("  Expiry: {}", exp);
            }
            if let Some(s) = &server {
                println!("  Server: {}", s);
            }
            if cloudflare_zone.is_some() || cloudflare_record.is_some() {
                println!();
                println!("  Cloudflare:");
                if let Some(z) = &cloudflare_zone {
                    println!("    Zone:   {}", z);
                }
                if let Some(r) = &cloudflare_record {
                    println!("    Record: {}", r);
                }
            }
        }

        DomainCommands::Show { domain } => {
            let dom = db
                .get_domain_by_name(&domain)
                .await?
                .or(db.get_domain(&domain).await?)
                .ok_or_else(|| anyhow::anyhow!("Domain '{}' not found", domain))?;

            let ssl_icon = if dom.ssl { "🔒" } else { "🔓" };

            println!();
            println!("  {} {}", ssl_icon, dom.domain);
            println!("  ─────────────────────────────");
            println!("  ID:     {}", dom.id);
            println!("  Type:   {}", dom.domain_type);
            println!("  SSL:    {}", if dom.ssl { "enabled" } else { "disabled" });
            if let Some(exp) = &dom.ssl_expiry {
                println!("  Expiry: {}", exp);
            }
            if let Some(s) = &dom.server_id {
                println!("  Server: {}", s);
            }
            if dom.cloudflare_zone_id.is_some() || dom.cloudflare_record_id.is_some() {
                println!();
                println!("  Cloudflare:");
                if let Some(z) = &dom.cloudflare_zone_id {
                    println!("    Zone:   {}", z);
                }
                if let Some(r) = &dom.cloudflare_record_id {
                    println!("    Record: {}", r);
                }
            }
            println!();
        }

        DomainCommands::Remove { domain } => {
            let dom = db
                .get_domain_by_name(&domain)
                .await?
                .or(db.get_domain(&domain).await?)
                .ok_or_else(|| anyhow::anyhow!("Domain '{}' not found", domain))?;

            if db.remove_domain(&dom.id).await? {
                println!("✓ Domain '{}' removed", dom.domain);
            }
        }
    }

    Ok(())
}
