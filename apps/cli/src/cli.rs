use crate::{
    Commands, CoolifyCommands, DatabaseCommands, DockerCommands, DomainCommands, GitCommands,
    ProjectCommands, ScriptCommands, ServerCommands, SshCommands,
};
use pctrl_coolify::CoolifyManager;
use pctrl_core::{
    AuthMethod, Config, CoolifyInstance, DatabaseCredentials, DatabaseType, DockerHost, Domain,
    DomainType, GitRepo, Project, ProjectResource, ProjectStatus, ResourceType, Script, ScriptType,
    Server, ServerType, SshConnection,
};
use pctrl_database::Database;
use pctrl_docker::DockerManager;
use pctrl_git::GitManager;
use pctrl_ssh::SshManager;
use std::sync::Arc;

pub async fn handle_command(
    command: Commands,
    config: Arc<Config>,
    db: Arc<Database>,
) -> anyhow::Result<()> {
    match command {
        // v6: Project-centric commands
        Commands::Project { command } => handle_project_command(command, &db).await,
        Commands::Server { command } => handle_server_command(command, &db).await,
        Commands::Domain { command } => handle_domain_command(command, &db).await,
        Commands::Database { command } => handle_database_command(command, &db).await,
        Commands::Script { command } => handle_script_command(command, &db).await,
        // Legacy commands
        Commands::Ssh { command } => handle_ssh_command(command, &config, &db).await,
        Commands::Docker { command } => handle_docker_command(command, &config, &db).await,
        Commands::Coolify { command } => handle_coolify_command(command, &config, &db).await,
        Commands::Git { command } => handle_git_command(command, &config, &db).await,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// v6: PROJECT COMMAND HANDLER
// ═══════════════════════════════════════════════════════════════════════════════

async fn handle_project_command(command: ProjectCommands, db: &Database) -> anyhow::Result<()> {
    match command {
        ProjectCommands::List => {
            let projects = db.list_projects().await?;
            if projects.is_empty() {
                println!("No projects configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl project add <name> [-d description] [-s stack]");
            } else {
                println!("Projects ({}):", projects.len());
                println!();
                for project in projects {
                    let status_icon = match project.status {
                        ProjectStatus::Live => "🟢",
                        ProjectStatus::Staging => "🟡",
                        ProjectStatus::Dev => "🔵",
                        ProjectStatus::Archived => "⚫",
                    };
                    let stack_str = if project.stack.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", project.stack.join(", "))
                    };
                    println!(
                        "  {} {} - {}{}",
                        status_icon, project.name, project.status, stack_str
                    );
                }
            }
        }

        ProjectCommands::Add {
            name,
            description,
            stack,
            status,
        } => {
            let id = name.to_lowercase().replace(' ', "-");

            if db.get_project_by_name(&name).await?.is_some() {
                anyhow::bail!("Project '{}' already exists.", name);
            }

            let stack_vec: Vec<String> = stack
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let status: ProjectStatus = status.parse().unwrap_or_default();

            let project = Project {
                id: id.clone(),
                name: name.clone(),
                description,
                stack: stack_vec.clone(),
                status: status.clone(),
                color: None,
                icon: None,
                notes: None,
            };

            db.save_project(&project).await?;

            println!("✓ Project added:");
            println!();
            println!("  Name:   {}", name);
            println!("  ID:     {}", id);
            println!("  Status: {}", status);
            if !stack_vec.is_empty() {
                println!("  Stack:  {}", stack_vec.join(", "));
            }
        }

        ProjectCommands::Show { name } => {
            let project = db
                .get_project_by_name(&name)
                .await?
                .or(db.get_project(&name).await?)
                .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", name))?;

            let status_icon = match project.status {
                ProjectStatus::Live => "🟢",
                ProjectStatus::Staging => "🟡",
                ProjectStatus::Dev => "🔵",
                ProjectStatus::Archived => "⚫",
            };

            println!();
            println!("  {} {}", status_icon, project.name);
            println!("  ─────────────────────────────");
            println!("  ID:     {}", project.id);
            println!("  Status: {}", project.status);
            if !project.stack.is_empty() {
                println!("  Stack:  {}", project.stack.join(", "));
            }
            if let Some(desc) = &project.description {
                println!("  Desc:   {}", desc);
            }

            // Show linked resources
            let resources = db.get_project_resources(&project.id).await?;
            if !resources.is_empty() {
                println!();
                println!("  Resources ({}):", resources.len());
                for res in resources {
                    let role_str = res.role.map(|r| format!(" ({})", r)).unwrap_or_default();
                    println!(
                        "    {} {} → {}{}",
                        res.resource_type, res.resource_id, res.id, role_str
                    );
                }
            }
            println!();
        }

        ProjectCommands::Remove { name } => {
            let project = db
                .get_project_by_name(&name)
                .await?
                .or(db.get_project(&name).await?)
                .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", name))?;

            if db.remove_project(&project.id).await? {
                println!("✓ Project '{}' removed", project.name);
            }
        }

        ProjectCommands::Link {
            project,
            resource_type,
            resource_id,
            role,
        } => {
            let proj = db
                .get_project_by_name(&project)
                .await?
                .or(db.get_project(&project).await?)
                .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project))?;

            let res_type: ResourceType = resource_type
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;

            let link = ProjectResource {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: proj.id.clone(),
                resource_type: res_type.clone(),
                resource_id: resource_id.clone(),
                role: role.clone(),
                notes: None,
            };

            db.link_project_resource(&link).await?;

            println!(
                "✓ Linked {} '{}' to project '{}'",
                res_type, resource_id, proj.name
            );
            if let Some(r) = role {
                println!("  Role: {}", r);
            }
        }

        ProjectCommands::Unlink { project, link_id } => {
            let proj = db
                .get_project_by_name(&project)
                .await?
                .or(db.get_project(&project).await?)
                .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project))?;

            if db.unlink_project_resource(&link_id).await? {
                println!("✓ Unlinked resource from project '{}'", proj.name);
            } else {
                println!("✗ Link '{}' not found", link_id);
            }
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// v6: SERVER COMMAND HANDLER
// ═══════════════════════════════════════════════════════════════════════════════

async fn handle_server_command(command: ServerCommands, db: &Database) -> anyhow::Result<()> {
    match command {
        ServerCommands::List => {
            let servers = db.list_servers().await?;
            if servers.is_empty() {
                println!("No servers configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl server add <name> <host> [-t type] [-p provider]");
            } else {
                println!("Servers ({}):", servers.len());
                println!();
                for server in servers {
                    let provider_str = server
                        .provider
                        .map(|p| format!(" ({})", p))
                        .unwrap_or_default();
                    println!(
                        "  🖥️  {} - {} [{}]{}",
                        server.name, server.host, server.server_type, provider_str
                    );
                }
            }
        }

        ServerCommands::Add {
            name,
            host,
            server_type,
            provider,
            ssh,
            location,
        } => {
            let id = name.to_lowercase().replace(' ', "-");

            if db.get_server_by_name(&name).await?.is_some() {
                anyhow::bail!("Server '{}' already exists.", name);
            }

            let server_type: ServerType = server_type.parse().unwrap_or_default();

            let server = Server {
                id: id.clone(),
                name: name.clone(),
                host: host.clone(),
                server_type: server_type.clone(),
                provider: provider.clone(),
                ssh_connection_id: ssh,
                location,
                specs: None,
                notes: None,
            };

            db.save_server(&server).await?;

            println!("✓ Server added:");
            println!();
            println!("  Name:     {}", name);
            println!("  ID:       {}", id);
            println!("  Host:     {}", host);
            println!("  Type:     {}", server_type);
            if let Some(p) = provider {
                println!("  Provider: {}", p);
            }
        }

        ServerCommands::Show { name } => {
            let server = db
                .get_server_by_name(&name)
                .await?
                .or(db.get_server(&name).await?)
                .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", name))?;

            println!();
            println!("  🖥️  {}", server.name);
            println!("  ─────────────────────────────");
            println!("  ID:       {}", server.id);
            println!("  Host:     {}", server.host);
            println!("  Type:     {}", server.server_type);
            if let Some(p) = &server.provider {
                println!("  Provider: {}", p);
            }
            if let Some(l) = &server.location {
                println!("  Location: {}", l);
            }
            if let Some(ssh) = &server.ssh_connection_id {
                println!("  SSH:      {}", ssh);
            }
            println!();
        }

        ServerCommands::Remove { name } => {
            let server = db
                .get_server_by_name(&name)
                .await?
                .or(db.get_server(&name).await?)
                .ok_or_else(|| anyhow::anyhow!("Server '{}' not found", name))?;

            if db.remove_server(&server.id).await? {
                println!("✓ Server '{}' removed", server.name);
            }
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// v6: DOMAIN COMMAND HANDLER
// ═══════════════════════════════════════════════════════════════════════════════

async fn handle_domain_command(command: DomainCommands, db: &Database) -> anyhow::Result<()> {
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
                ssl_expiry: None,
                cloudflare_zone_id: None,
                cloudflare_record_id: None,
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
            if let Some(s) = server {
                println!("  Server: {}", s);
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

// ═══════════════════════════════════════════════════════════════════════════════
// v6: DATABASE COMMAND HANDLER
// ═══════════════════════════════════════════════════════════════════════════════

async fn handle_database_command(command: DatabaseCommands, db: &Database) -> anyhow::Result<()> {
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
                server_id: None,
                container_id: None,
                notes: None,
            };

            db.save_database_credentials(&creds).await?;

            println!("✓ Database credentials added:");
            println!();
            println!("  Name: {}", name);
            println!("  ID:   {}", id);
            println!("  Type: {}", db_type);
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

// ═══════════════════════════════════════════════════════════════════════════════
// v6: SCRIPT COMMAND HANDLER
// ═══════════════════════════════════════════════════════════════════════════════

async fn handle_script_command(command: ScriptCommands, db: &Database) -> anyhow::Result<()> {
    match command {
        ScriptCommands::List => {
            let scripts = db.list_scripts().await?;
            if scripts.is_empty() {
                println!("No scripts configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl script add <name> -c <command> [-s server]");
            } else {
                println!("Scripts ({}):", scripts.len());
                println!();
                for script in scripts {
                    let danger_icon = if script.dangerous { "⚠️ " } else { "" };
                    println!(
                        "  📜 {}{} [{}]",
                        danger_icon, script.name, script.script_type
                    );
                }
            }
        }

        ScriptCommands::Add {
            name,
            command,
            description,
            script_type,
            server,
            project,
            dangerous,
        } => {
            let id = name.to_lowercase().replace(' ', "-");

            let script_type: ScriptType = script_type.parse().unwrap_or_default();

            let script = Script {
                id: id.clone(),
                name: name.clone(),
                description,
                command: command.clone(),
                script_type: script_type.clone(),
                server_id: server,
                project_id: project,
                dangerous,
                last_run: None,
                last_result: None,
            };

            db.save_script(&script).await?;

            println!("✓ Script added:");
            println!();
            println!("  Name:    {}", name);
            println!("  ID:      {}", id);
            println!("  Type:    {}", script_type);
            println!("  Command: {}", command);
            if dangerous {
                println!("  ⚠️  Marked as dangerous");
            }
        }

        ScriptCommands::Show { name } => {
            let script = db
                .get_script(&name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Script '{}' not found", name))?;

            let danger_icon = if script.dangerous { "⚠️ " } else { "" };

            println!();
            println!("  📜 {}{}", danger_icon, script.name);
            println!("  ─────────────────────────────");
            println!("  ID:      {}", script.id);
            println!("  Type:    {}", script.script_type);
            println!("  Command: {}", script.command);
            if let Some(desc) = &script.description {
                println!("  Desc:    {}", desc);
            }
            if let Some(server) = &script.server_id {
                println!("  Server:  {}", server);
            }
            if let Some(project) = &script.project_id {
                println!("  Project: {}", project);
            }
            println!();
        }

        ScriptCommands::Run { name, force } => {
            let script = db
                .get_script(&name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Script '{}' not found", name))?;

            if script.dangerous && !force {
                println!("⚠️  This script is marked as dangerous!");
                println!("    Command: {}", script.command);
                println!();
                println!("Use --force to run anyway.");
                return Ok(());
            }

            println!("Running script '{}'...", script.name);
            println!("Command: {}", script.command);
            println!();
            println!("(Script execution not yet implemented)");
        }

        ScriptCommands::Remove { name } => {
            if db.remove_script(&name).await? {
                println!("✓ Script '{}' removed", name);
            } else {
                println!("✗ Script '{}' not found", name);
            }
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// LEGACY COMMAND HANDLERS
// ═══════════════════════════════════════════════════════════════════════════════

async fn handle_ssh_command(
    command: SshCommands,
    config: &Config,
    db: &Database,
) -> anyhow::Result<()> {
    // ─────────────────────────────────────────────────────────────────────────
    // Manager mit Config-Daten initialisieren
    // ─────────────────────────────────────────────────────────────────────────
    let mut ssh_manager = SshManager::new();
    for conn in &config.ssh_connections {
        ssh_manager.add_connection(conn.clone());
    }

    match command {
        SshCommands::List => {
            let connections = ssh_manager.list_connections();
            if connections.is_empty() {
                println!("No SSH connections configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl ssh add <name> <host> -u <user> [-k ~/.ssh/id_rsa]");
            } else {
                println!("SSH Connections ({}):", connections.len());
                println!();
                for conn in connections {
                    let auth_icon = match &conn.auth_method {
                        AuthMethod::PublicKey { .. } => "🔑",
                        AuthMethod::Password => "🔒",
                    };
                    println!(
                        "  {} [{}] {} - {}@{}:{}",
                        auth_icon, conn.id, conn.name, conn.username, conn.host, conn.port
                    );
                }
            }
        }

        SshCommands::Add {
            name,
            host,
            user,
            port,
            key,
        } => {
            // ID = name (lowercase, keine Leerzeichen)
            let id = name.to_lowercase().replace(' ', "-");

            // Prüfen ob schon existiert
            if db.ssh_connection_exists(&id).await? {
                anyhow::bail!("Connection '{}' already exists. Use a different name.", id);
            }

            // Default Key-Pfad
            let key_path = key.unwrap_or_else(|| {
                dirs::home_dir()
                    .map(|h| h.join(".ssh").join("id_rsa").to_string_lossy().to_string())
                    .unwrap_or_else(|| "~/.ssh/id_rsa".to_string())
            });

            let connection = SshConnection {
                id: id.clone(),
                name: name.clone(),
                host: host.clone(),
                port,
                username: user.clone(),
                auth_method: AuthMethod::PublicKey { key_path },
            };

            // In DB speichern
            db.save_ssh_connection(&connection).await?;

            println!("✓ SSH connection added:");
            println!();
            println!("  Name:     {}", name);
            println!("  ID:       {}", id);
            println!("  Host:     {}:{}", host, port);
            println!("  User:     {}", user);
            println!();
            println!("Test with: pctrl ssh connect {}", id);
        }

        SshCommands::Remove { id } => {
            if db.remove_ssh_connection(&id).await? {
                println!("✓ SSH connection '{}' removed", id);
            } else {
                println!("✗ Connection '{}' not found", id);
            }
        }

        SshCommands::Connect { id } => {
            println!("Connecting to SSH host: {}", id);
            let _session = ssh_manager.connect(&id)?;
            println!("✓ Connected successfully");
        }

        SshCommands::Exec { id, command } => {
            println!("Executing on {}: {}", id, command);
            let output = ssh_manager.execute_command(&id, &command)?;
            println!("{}", output);
        }
    }

    Ok(())
}

async fn handle_docker_command(
    command: DockerCommands,
    config: &Config,
    db: &Database,
) -> anyhow::Result<()> {
    // ─────────────────────────────────────────────────────────────────────────
    // Manager mit Config-Daten initialisieren
    // ─────────────────────────────────────────────────────────────────────────
    let mut docker_manager = DockerManager::new();
    for host in &config.docker_hosts {
        docker_manager.add_host(host.clone());
    }

    match command {
        DockerCommands::Hosts => {
            if config.docker_hosts.is_empty() {
                println!("No Docker hosts configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl docker add <name> [-u <url>]");
            } else {
                println!("Docker Hosts ({}):", config.docker_hosts.len());
                println!();
                for host in &config.docker_hosts {
                    println!("  🐳 [{}] {} - {}", host.id, host.name, host.url);
                }
            }
        }

        DockerCommands::Add { name, url } => {
            let id = name.to_lowercase().replace(' ', "-");

            if db.docker_host_exists(&id).await? {
                anyhow::bail!("Docker host '{}' already exists. Use a different name.", id);
            }

            let host = DockerHost {
                id: id.clone(),
                name: name.clone(),
                url: url.clone(),
            };

            db.save_docker_host(&host).await?;

            println!("✓ Docker host added:");
            println!();
            println!("  Name:  {}", name);
            println!("  ID:    {}", id);
            println!("  URL:   {}", url);
            println!();
            println!("List containers with: pctrl docker list {}", id);
        }

        DockerCommands::Remove { id } => {
            if db.remove_docker_host(&id).await? {
                println!("✓ Docker host '{}' removed", id);
            } else {
                println!("✗ Docker host '{}' not found", id);
            }
        }

        DockerCommands::List { host_id } => {
            let containers = docker_manager.list_containers(&host_id).await?;
            if containers.is_empty() {
                println!("No containers on host {}", host_id);
            } else {
                println!("Containers on {} ({}):", host_id, containers.len());
                println!();
                for container in containers {
                    let state_icon = match container.state.as_str() {
                        "running" => "●",
                        "exited" => "○",
                        _ => "◌",
                    };
                    println!(
                        "  {} [{}] {} - {}",
                        state_icon,
                        &container.id[..12],
                        container.name,
                        container.image
                    );
                }
            }
        }
        DockerCommands::Start {
            host_id,
            container_id,
        } => {
            docker_manager
                .start_container(&host_id, &container_id)
                .await?;
            println!("✓ Container {} started", container_id);
        }
        DockerCommands::Stop {
            host_id,
            container_id,
        } => {
            docker_manager
                .stop_container(&host_id, &container_id)
                .await?;
            println!("✓ Container {} stopped", container_id);
        }
    }

    Ok(())
}

async fn handle_coolify_command(
    command: CoolifyCommands,
    config: &Config,
    db: &Database,
) -> anyhow::Result<()> {
    // ─────────────────────────────────────────────────────────────────────────
    // Manager mit Config-Daten initialisieren
    // ─────────────────────────────────────────────────────────────────────────
    let mut coolify_manager = CoolifyManager::new();
    for instance in &config.coolify_instances {
        coolify_manager.add_instance(instance.clone());
    }

    match command {
        CoolifyCommands::Instances => {
            if config.coolify_instances.is_empty() {
                println!("No Coolify instances configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl coolify add <name> -u <url> -t <token>");
            } else {
                println!("Coolify Instances ({}):", config.coolify_instances.len());
                println!();
                for instance in &config.coolify_instances {
                    println!(
                        "  🚀 [{}] {} - {}",
                        instance.id, instance.name, instance.url
                    );
                }
            }
        }

        CoolifyCommands::Add { name, url, token } => {
            let id = name.to_lowercase().replace(' ', "-");

            if db.coolify_instance_exists(&id).await? {
                anyhow::bail!(
                    "Coolify instance '{}' already exists. Use a different name.",
                    id
                );
            }

            let instance = CoolifyInstance {
                id: id.clone(),
                name: name.clone(),
                url: url.clone(),
                api_key: token,
            };

            db.save_coolify_instance(&instance).await?;

            println!("✓ Coolify instance added:");
            println!();
            println!("  Name:  {}", name);
            println!("  ID:    {}", id);
            println!("  URL:   {}", url);
            println!();
            println!("List deployments with: pctrl coolify list {}", id);
        }

        CoolifyCommands::Remove { id } => {
            if db.remove_coolify_instance(&id).await? {
                println!("✓ Coolify instance '{}' removed", id);
            } else {
                println!("✗ Coolify instance '{}' not found", id);
            }
        }

        CoolifyCommands::List { instance_id } => {
            let deployments = coolify_manager.list_deployments(&instance_id).await?;
            if deployments.is_empty() {
                println!("No deployments on instance {}", instance_id);
            } else {
                println!("Deployments on {} ({}):", instance_id, deployments.len());
                println!();
                for deployment in deployments {
                    let status_icon = match deployment.status.as_str() {
                        "running" | "healthy" => "●",
                        "stopped" | "exited" => "○",
                        "error" | "failed" => "✗",
                        _ => "◌",
                    };
                    println!(
                        "  {} [{}] {} - {}",
                        status_icon, deployment.id, deployment.name, deployment.status
                    );
                }
            }
        }
        CoolifyCommands::Deploy {
            instance_id,
            project_id,
        } => {
            coolify_manager
                .deploy_project(&instance_id, &project_id)
                .await?;
            println!("✓ Deployment started for project {}", project_id);
        }
    }

    Ok(())
}

async fn handle_git_command(
    command: GitCommands,
    config: &Config,
    db: &Database,
) -> anyhow::Result<()> {
    // ─────────────────────────────────────────────────────────────────────────
    // Manager mit Config-Daten initialisieren
    // ─────────────────────────────────────────────────────────────────────────
    let mut git_manager = GitManager::new();
    for repo in &config.git_repos {
        git_manager.add_repo(repo.clone());
    }

    match command {
        GitCommands::Repos => {
            if config.git_repos.is_empty() {
                println!("No Git repositories configured.");
                println!();
                println!("Add one with:");
                println!("  pctrl git add <name> -p <path>");
            } else {
                println!("Git Repositories ({}):", config.git_repos.len());
                println!();
                for repo in &config.git_repos {
                    println!("  📁 [{}] {} - {}", repo.id, repo.name, repo.path);
                }
            }
        }

        GitCommands::Add { name, path } => {
            let id = name.to_lowercase().replace(' ', "-");

            if db.git_repo_exists(&id).await? {
                anyhow::bail!(
                    "Git repository '{}' already exists. Use a different name.",
                    id
                );
            }

            // Verify path exists
            let abs_path = std::path::Path::new(&path);
            if !abs_path.exists() {
                anyhow::bail!("Path '{}' does not exist.", path);
            }

            let repo = GitRepo {
                id: id.clone(),
                name: name.clone(),
                path: path.clone(),
                remote_url: None,
            };

            db.save_git_repo(&repo).await?;

            println!("✓ Git repository added:");
            println!();
            println!("  Name:  {}", name);
            println!("  ID:    {}", id);
            println!("  Path:  {}", path);
            println!();
            println!("List releases with: pctrl git list {}", id);
        }

        GitCommands::Remove { id } => {
            if db.remove_git_repo(&id).await? {
                println!("✓ Git repository '{}' removed", id);
            } else {
                println!("✗ Git repository '{}' not found", id);
            }
        }

        GitCommands::List { repo_id } => {
            let releases = git_manager.list_releases(&repo_id)?;
            if releases.is_empty() {
                println!("No releases in repository {}", repo_id);
            } else {
                println!("Releases in {} ({}):", repo_id, releases.len());
                println!();
                for release in releases {
                    println!("  [{}] {} - {}", release.tag, release.name, release.message);
                }
            }
        }
        GitCommands::Create {
            repo_id,
            tag,
            message,
        } => {
            git_manager.create_release(&repo_id, &tag, &message)?;
            println!("✓ Release {} created", tag);
        }
        GitCommands::Push { repo_id } => {
            git_manager.push_tags(&repo_id)?;
            println!("✓ Tags pushed to remote");
        }
    }

    Ok(())
}
