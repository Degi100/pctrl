use pctrl_core::{AuthMethod, Result, SshConnection};
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
}

impl Default for SshManager {
    fn default() -> Self {
        Self::new()
    }
}
