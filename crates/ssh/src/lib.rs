use pctrl_core::{AuthMethod, Result, ServerSpecs, SshConnection};
use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

/// SSH connection manager
pub struct SshManager {
    connections: Vec<SshConnection>,
}

impl SshManager {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
        }
    }

    /// Add a new SSH connection
    pub fn add_connection(&mut self, connection: SshConnection) {
        self.connections.push(connection);
    }

    /// Get a connection by ID
    pub fn get_connection(&self, id: &str) -> Option<&SshConnection> {
        self.connections.iter().find(|c| c.id == id)
    }

    /// Connect to an SSH host (with optional password for password auth)
    pub fn connect(&self, id: &str) -> Result<Session> {
        self.connect_with_password(id, None)
    }

    /// Connect to an SSH host with explicit password
    pub fn connect_with_password(&self, id: &str, password: Option<&str>) -> Result<Session> {
        let conn = self
            .connections
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| pctrl_core::Error::Ssh("Connection not found".to_string()))?;

        let tcp = TcpStream::connect(format!("{}:{}", conn.host, conn.port))
            .map_err(|e| pctrl_core::Error::Ssh(format!("TCP connection failed: {}", e)))?;

        let mut session = Session::new()
            .map_err(|e| pctrl_core::Error::Ssh(format!("Session creation failed: {}", e)))?;

        session.set_tcp_stream(tcp);
        session
            .handshake()
            .map_err(|e| pctrl_core::Error::Ssh(format!("SSH handshake failed: {}", e)))?;

        match &conn.auth_method {
            AuthMethod::Password => {
                let pw = password.ok_or_else(|| {
                    pctrl_core::Error::Ssh("Password required for authentication".to_string())
                })?;
                session.userauth_password(&conn.username, pw).map_err(|e| {
                    pctrl_core::Error::Ssh(format!("Password authentication failed: {}", e))
                })?;
            }
            AuthMethod::PublicKey { key_path } => {
                session
                    .userauth_pubkey_file(&conn.username, None, Path::new(key_path), None)
                    .map_err(|e| {
                        pctrl_core::Error::Ssh(format!("Public key authentication failed: {}", e))
                    })?;
            }
            AuthMethod::Key { path, passphrase } => {
                session
                    .userauth_pubkey_file(
                        &conn.username,
                        None,
                        Path::new(path),
                        passphrase.as_deref(),
                    )
                    .map_err(|e| {
                        pctrl_core::Error::Ssh(format!("Key authentication failed: {}", e))
                    })?;
            }
            AuthMethod::Agent => {
                let mut agent = session.agent().map_err(|e| {
                    pctrl_core::Error::Ssh(format!("Failed to get SSH agent: {}", e))
                })?;

                agent.connect().map_err(|e| {
                    pctrl_core::Error::Ssh(format!("Failed to connect to SSH agent: {}", e))
                })?;

                agent.list_identities().map_err(|e| {
                    pctrl_core::Error::Ssh(format!("Failed to list agent identities: {}", e))
                })?;

                // Try each identity until one works
                let mut authenticated = false;
                for identity in agent.identities().unwrap_or_default() {
                    if agent.userauth(&conn.username, &identity).is_ok() {
                        authenticated = true;
                        break;
                    }
                }

                if !authenticated {
                    return Err(pctrl_core::Error::Ssh(
                        "SSH agent authentication failed: no valid identity found".to_string(),
                    ));
                }
            }
        }

        Ok(session)
    }

    /// Test if a connection can be established (for health checks)
    pub fn test_connection(&self, id: &str, password: Option<&str>) -> Result<()> {
        let conn = self
            .connections
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| pctrl_core::Error::Ssh("Connection not found".to_string()))?;

        // Try TCP connection with timeout
        let addr = format!("{}:{}", conn.host, conn.port);
        let tcp = TcpStream::connect_timeout(
            &addr
                .parse()
                .map_err(|e| pctrl_core::Error::Ssh(format!("Invalid address: {}", e)))?,
            Duration::from_secs(5),
        )
        .map_err(|e| pctrl_core::Error::Ssh(format!("TCP connection failed: {}", e)))?;

        let mut session = Session::new()
            .map_err(|e| pctrl_core::Error::Ssh(format!("Session creation failed: {}", e)))?;

        session.set_tcp_stream(tcp);
        session
            .handshake()
            .map_err(|e| pctrl_core::Error::Ssh(format!("SSH handshake failed: {}", e)))?;

        // For public key auth, try to authenticate
        // For password auth without password provided, just check handshake succeeded
        match &conn.auth_method {
            AuthMethod::Password => {
                if let Some(pw) = password {
                    session.userauth_password(&conn.username, pw).map_err(|e| {
                        pctrl_core::Error::Ssh(format!("Password authentication failed: {}", e))
                    })?;
                }
                // If no password, just verify TCP + handshake works
            }
            AuthMethod::PublicKey { key_path } => {
                session
                    .userauth_pubkey_file(&conn.username, None, Path::new(key_path), None)
                    .map_err(|e| {
                        pctrl_core::Error::Ssh(format!("Public key authentication failed: {}", e))
                    })?;
            }
            AuthMethod::Key { path, passphrase } => {
                session
                    .userauth_pubkey_file(
                        &conn.username,
                        None,
                        Path::new(path),
                        passphrase.as_deref(),
                    )
                    .map_err(|e| {
                        pctrl_core::Error::Ssh(format!("Key authentication failed: {}", e))
                    })?;
            }
            AuthMethod::Agent => {
                let mut agent = session.agent().map_err(|e| {
                    pctrl_core::Error::Ssh(format!("Failed to get SSH agent: {}", e))
                })?;

                agent.connect().map_err(|e| {
                    pctrl_core::Error::Ssh(format!("Failed to connect to SSH agent: {}", e))
                })?;

                agent.list_identities().map_err(|e| {
                    pctrl_core::Error::Ssh(format!("Failed to list agent identities: {}", e))
                })?;

                let mut authenticated = false;
                for identity in agent.identities().unwrap_or_default() {
                    if agent.userauth(&conn.username, &identity).is_ok() {
                        authenticated = true;
                        break;
                    }
                }

                if !authenticated {
                    return Err(pctrl_core::Error::Ssh(
                        "SSH agent authentication failed".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Execute a command on a remote host
    pub fn execute_command(&self, id: &str, command: &str) -> Result<String> {
        self.execute_command_with_password(id, command, None)
    }

    /// Execute a command on a remote host with explicit password
    pub fn execute_command_with_password(
        &self,
        id: &str,
        command: &str,
        password: Option<&str>,
    ) -> Result<String> {
        let session = self.connect_with_password(id, password)?;

        let mut channel = session
            .channel_session()
            .map_err(|e| pctrl_core::Error::Ssh(format!("Channel creation failed: {}", e)))?;

        channel
            .exec(command)
            .map_err(|e| pctrl_core::Error::Ssh(format!("Command execution failed: {}", e)))?;

        let mut output = String::new();
        std::io::Read::read_to_string(&mut channel, &mut output)
            .map_err(|e| pctrl_core::Error::Ssh(format!("Failed to read output: {}", e)))?;

        channel
            .wait_close()
            .map_err(|e| pctrl_core::Error::Ssh(format!("Channel close failed: {}", e)))?;

        Ok(output)
    }

    /// List all connections
    pub fn list_connections(&self) -> &[SshConnection] {
        &self.connections
    }

    /// Detect server specs via SSH (CPU cores, RAM, Disk)
    pub fn detect_server_specs(&self, id: &str, password: Option<&str>) -> Result<ServerSpecs> {
        let session = self.connect_with_password(id, password)?;

        // Get CPU cores
        let cpu_cores = self
            .exec_on_session(
                &session,
                "nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null",
            )
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok());

        // Get RAM in GB
        let ram_gb = self
            .exec_on_session(
                &session,
                "free -g 2>/dev/null | awk '/^Mem:/{print $2}' || sysctl -n hw.memsize 2>/dev/null | awk '{print int($1/1024/1024/1024)}'",
            )
            .ok()
            .and_then(|s| s.trim().parse::<u16>().ok());

        // Get Disk in GB (root partition)
        let disk_gb = self
            .exec_on_session(
                &session,
                "df -BG / 2>/dev/null | awk 'NR==2{gsub(/G/,\"\",$2); print $2}' || df -g / 2>/dev/null | awk 'NR==2{print $2}'",
            )
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok());

        Ok(ServerSpecs {
            cpu_cores,
            ram_gb,
            disk_gb,
        })
    }

    /// Execute command on an existing session
    fn exec_on_session(&self, session: &Session, command: &str) -> Result<String> {
        let mut channel = session
            .channel_session()
            .map_err(|e| pctrl_core::Error::Ssh(format!("Channel creation failed: {}", e)))?;

        channel
            .exec(command)
            .map_err(|e| pctrl_core::Error::Ssh(format!("Command execution failed: {}", e)))?;

        let mut output = String::new();
        std::io::Read::read_to_string(&mut channel, &mut output)
            .map_err(|e| pctrl_core::Error::Ssh(format!("Failed to read output: {}", e)))?;

        channel
            .wait_close()
            .map_err(|e| pctrl_core::Error::Ssh(format!("Channel close failed: {}", e)))?;

        Ok(output)
    }
}

impl Default for SshManager {
    fn default() -> Self {
        Self::new()
    }
}
