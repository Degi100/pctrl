//! Credential types for secure storage of SSH keys, API tokens, etc.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Credential - secure storage for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: String,
    pub name: String,
    pub credential_type: CredentialType,
    pub data: CredentialData,
    pub notes: Option<String>,
}

/// Type of credential
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum CredentialType {
    #[default]
    SshKey,
    SshAgent,
    ApiToken,
    BasicAuth,
    OAuth,
}

/// Credential data - varies by type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CredentialData {
    /// SSH Key authentication
    SshKey {
        username: String,
        #[serde(default = "default_ssh_port")]
        port: u16,
        key_path: String,
        passphrase: Option<String>,
    },
    /// SSH Agent authentication (uses system SSH agent)
    SshAgent {
        username: String,
        #[serde(default = "default_ssh_port")]
        port: u16,
    },
    /// API Token (Bearer token)
    ApiToken { token: String, url: Option<String> },
    /// Basic Auth (username/password)
    BasicAuth {
        username: String,
        password: String,
        url: Option<String>,
    },
    /// OAuth tokens
    OAuth {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<String>,
        url: Option<String>,
    },
}

fn default_ssh_port() -> u16 {
    22
}

impl fmt::Display for CredentialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialType::SshKey => write!(f, "ssh_key"),
            CredentialType::SshAgent => write!(f, "ssh_agent"),
            CredentialType::ApiToken => write!(f, "api_token"),
            CredentialType::BasicAuth => write!(f, "basic_auth"),
            CredentialType::OAuth => write!(f, "oauth"),
        }
    }
}

impl std::str::FromStr for CredentialType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ssh" | "ssh_key" | "sshkey" => Ok(CredentialType::SshKey),
            "agent" | "ssh_agent" | "sshagent" => Ok(CredentialType::SshAgent),
            "api" | "api_token" | "apitoken" | "token" => Ok(CredentialType::ApiToken),
            "basic" | "basic_auth" | "basicauth" => Ok(CredentialType::BasicAuth),
            "oauth" => Ok(CredentialType::OAuth),
            _ => Err(format!("Unknown credential type: {}", s)),
        }
    }
}

impl Credential {
    /// Create a new SSH key credential
    pub fn new_ssh(
        id: String,
        name: String,
        username: String,
        key_path: String,
        port: Option<u16>,
        passphrase: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            credential_type: CredentialType::SshKey,
            data: CredentialData::SshKey {
                username,
                port: port.unwrap_or(22),
                key_path,
                passphrase,
            },
            notes: None,
        }
    }

    /// Create a new API token credential
    pub fn new_api_token(id: String, name: String, token: String, url: Option<String>) -> Self {
        Self {
            id,
            name,
            credential_type: CredentialType::ApiToken,
            data: CredentialData::ApiToken { token, url },
            notes: None,
        }
    }

    /// Create a new basic auth credential
    pub fn new_basic_auth(
        id: String,
        name: String,
        username: String,
        password: String,
        url: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            credential_type: CredentialType::BasicAuth,
            data: CredentialData::BasicAuth {
                username,
                password,
                url,
            },
            notes: None,
        }
    }

    /// Get SSH details if this is an SSH credential
    pub fn as_ssh(&self) -> Option<(&str, u16, &str, Option<&str>)> {
        match &self.data {
            CredentialData::SshKey {
                username,
                port,
                key_path,
                passphrase,
            } => Some((username, *port, key_path, passphrase.as_deref())),
            _ => None,
        }
    }

    /// Get API token if this is an API token credential
    pub fn as_api_token(&self) -> Option<(&str, Option<&str>)> {
        match &self.data {
            CredentialData::ApiToken { token, url } => Some((token, url.as_deref())),
            _ => None,
        }
    }
}
