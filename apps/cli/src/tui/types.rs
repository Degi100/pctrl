//! TUI type definitions

#[derive(Clone, Copy, PartialEq)]
pub enum SelectedPanel {
    Status,
    Projects,
    Servers,
    Domains,
    Databases,
    Scripts,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Adding,
}

#[derive(Clone, Default)]
pub struct InputForm {
    // Common
    pub name: String,
    pub description: String,
    pub current_field: usize,
    pub message: Option<String>,
    // Project
    pub stack: String,
    pub status: String,
    // Server
    pub host: String,
    pub server_type: String,
    pub provider: String,
    // Domain
    pub domain: String,
    pub domain_type: String,
    pub ssl: String,
    // Database
    pub db_type: String,
    pub port: String,
    pub user: String,
    pub password: String,
    // Script
    pub command: String,
    pub script_type: String,
}
