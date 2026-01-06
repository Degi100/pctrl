use pctrl_core::{AuthMethod, Result, SshConnection};
use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;

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

    /// Connect to an SSH host
    pub fn connect(&self, id: &str) -> Result<Session> {
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
                // TODO: Implement password authentication
                // In production, retrieve password securely from keychain/credential manager
                return Err(pctrl_core::Error::Ssh(
                    "Password authentication not yet implemented. Please use public key authentication.".to_string(),
                ));
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

    /// Execute a command on a remote host
    pub fn execute_command(&self, id: &str, command: &str) -> Result<String> {
        let session = self.connect(id)?;

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
