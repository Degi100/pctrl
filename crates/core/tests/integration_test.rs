use pctrl_core::{Config, Mode, SshConnection, AuthMethod};

#[test]
fn test_config_default() {
    let config = Config::default();
    assert_eq!(config.database_path, "pctrl.db");
    assert_eq!(config.ssh_connections.len(), 0);
    assert_eq!(config.docker_hosts.len(), 0);
    assert_eq!(config.coolify_instances.len(), 0);
    assert_eq!(config.git_repos.len(), 0);
}

#[test]
fn test_ssh_connection_creation() {
    let conn = SshConnection {
        id: "test-1".to_string(),
        name: "Test Server".to_string(),
        host: "example.com".to_string(),
        port: 22,
        username: "user".to_string(),
        auth_method: AuthMethod::PublicKey {
            key_path: "/path/to/key".to_string(),
        },
    };
    
    assert_eq!(conn.id, "test-1");
    assert_eq!(conn.host, "example.com");
    assert_eq!(conn.port, 22);
}

#[test]
fn test_mode_display() {
    assert_eq!(Mode::Cli.to_string(), "cli");
    assert_eq!(Mode::Tui.to_string(), "tui");
    assert_eq!(Mode::Gui.to_string(), "gui");
}

#[test]
fn test_mode_equality() {
    assert_eq!(Mode::Cli, Mode::Cli);
    assert_ne!(Mode::Cli, Mode::Tui);
    assert_ne!(Mode::Tui, Mode::Gui);
}

#[test]
fn test_config_serialization() {
    let config = Config::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: Config = serde_json::from_str(&json).unwrap();
    
    assert_eq!(config.database_path, deserialized.database_path);
}
