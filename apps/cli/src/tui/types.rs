//! TUI type definitions

#[derive(Clone, Copy, PartialEq)]
pub enum SelectedPanel {
    Status,
    Projects,
    Ssh,
    Docker,
    Coolify,
    Git,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ConnectionStatus {
    Unknown,
    Online,
    Offline,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Adding,
}

#[derive(Clone, Default)]
pub struct InputForm {
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: String,
    pub url: String,
    pub path: String,
    pub token: String,
    pub description: String,
    pub stack: String,
    pub status: String,
    pub current_field: usize,
    pub message: Option<String>,
}
