use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use pctrl_core::{AuthMethod, Config, CoolifyInstance, DockerHost, GitRepo, SshConnection};
use pctrl_database::Database;
use uuid::Uuid;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq)]
enum SelectedPanel {
    Status,
    Ssh,
    Docker,
    Coolify,
    Git,
}

#[derive(Clone, Copy, PartialEq)]
enum ConnectionStatus {
    Unknown,    // Not yet tested (yellow)
    Online,     // Connection successful (green)
    Offline,    // Connection failed (red)
}

#[derive(Clone, Copy, PartialEq)]
enum InputMode {
    Normal,     // Normal navigation mode
    Adding,     // Adding a new entry (form input)
}

#[derive(Clone, Default)]
struct InputForm {
    name: String,
    host: String,
    user: String,
    port: String,
    url: String,
    path: String,
    token: String,
    current_field: usize,
    message: Option<String>,
}

struct App {
    selected_panel: SelectedPanel,
    config: Arc<Config>,
    db: Arc<Database>,
    // Connection status tracking: id -> status
    ssh_status: HashMap<String, ConnectionStatus>,
    docker_status: HashMap<String, ConnectionStatus>,
    coolify_status: HashMap<String, ConnectionStatus>,
    git_status: HashMap<String, ConnectionStatus>,
    // Input mode
    input_mode: InputMode,
    input_form: InputForm,
}

impl App {
    fn new(config: Arc<Config>, db: Arc<Database>) -> Self {
        // Initialize with Unknown status for all connections
        let ssh_status: HashMap<String, ConnectionStatus> = config
            .ssh_connections
            .iter()
            .map(|c| (c.id.clone(), ConnectionStatus::Unknown))
            .collect();
        let docker_status: HashMap<String, ConnectionStatus> = config
            .docker_hosts
            .iter()
            .map(|h| (h.id.clone(), ConnectionStatus::Unknown))
            .collect();
        let coolify_status: HashMap<String, ConnectionStatus> = config
            .coolify_instances
            .iter()
            .map(|i| (i.id.clone(), ConnectionStatus::Unknown))
            .collect();
        let git_status: HashMap<String, ConnectionStatus> = config
            .git_repos
            .iter()
            .map(|r| (r.id.clone(), ConnectionStatus::Unknown))
            .collect();

        Self {
            selected_panel: SelectedPanel::Status,
            config,
            db,
            ssh_status,
            docker_status,
            coolify_status,
            git_status,
            input_mode: InputMode::Normal,
            input_form: InputForm::default(),
        }
    }

    /// Get the fields for the current panel type
    fn get_form_fields(&self) -> Vec<(&'static str, &str)> {
        match self.selected_panel {
            SelectedPanel::Ssh => vec![
                ("Name", &self.input_form.name),
                ("Host", &self.input_form.host),
                ("User", &self.input_form.user),
                ("Port", &self.input_form.port),
            ],
            SelectedPanel::Docker => vec![
                ("Name", &self.input_form.name),
                ("URL", &self.input_form.url),
            ],
            SelectedPanel::Coolify => vec![
                ("Name", &self.input_form.name),
                ("URL", &self.input_form.url),
                ("Token", &self.input_form.token),
            ],
            SelectedPanel::Git => vec![
                ("Name", &self.input_form.name),
                ("Path", &self.input_form.path),
            ],
            SelectedPanel::Status => vec![],
        }
    }

    /// Get mutable reference to current input field
    fn current_input_mut(&mut self) -> Option<&mut String> {
        let field_idx = self.input_form.current_field;
        match self.selected_panel {
            SelectedPanel::Ssh => match field_idx {
                0 => Some(&mut self.input_form.name),
                1 => Some(&mut self.input_form.host),
                2 => Some(&mut self.input_form.user),
                3 => Some(&mut self.input_form.port),
                _ => None,
            },
            SelectedPanel::Docker => match field_idx {
                0 => Some(&mut self.input_form.name),
                1 => Some(&mut self.input_form.url),
                _ => None,
            },
            SelectedPanel::Coolify => match field_idx {
                0 => Some(&mut self.input_form.name),
                1 => Some(&mut self.input_form.url),
                2 => Some(&mut self.input_form.token),
                _ => None,
            },
            SelectedPanel::Git => match field_idx {
                0 => Some(&mut self.input_form.name),
                1 => Some(&mut self.input_form.path),
                _ => None,
            },
            SelectedPanel::Status => None,
        }
    }

    /// Get the number of fields for current panel
    fn field_count(&self) -> usize {
        match self.selected_panel {
            SelectedPanel::Ssh => 4,
            SelectedPanel::Docker => 2,
            SelectedPanel::Coolify => 3,
            SelectedPanel::Git => 2,
            SelectedPanel::Status => 0,
        }
    }

    /// Reset the input form
    fn reset_form(&mut self) {
        self.input_form = InputForm::default();
        // Set default port for SSH
        if self.selected_panel == SelectedPanel::Ssh {
            self.input_form.port = "22".to_string();
        }
        // Set default URL for Docker
        if self.selected_panel == SelectedPanel::Docker {
            self.input_form.url = "unix:///var/run/docker.sock".to_string();
        }
    }

    /// Check all connections and update their status
    fn check_all_connections(&mut self) {
        // Check Git repos (simple path existence check)
        for repo in &self.config.git_repos {
            let status = if Path::new(&repo.path).exists() {
                ConnectionStatus::Online
            } else {
                ConnectionStatus::Offline
            };
            self.git_status.insert(repo.id.clone(), status);
        }

        // Check Docker hosts (basic URL validation)
        for host in &self.config.docker_hosts {
            let status = if host.url.starts_with("unix://") {
                // Check if socket exists
                let socket_path = host.url.trim_start_matches("unix://");
                if Path::new(socket_path).exists() {
                    ConnectionStatus::Online
                } else {
                    ConnectionStatus::Offline
                }
            } else {
                // For TCP URLs, mark as Unknown (would need async check)
                ConnectionStatus::Unknown
            };
            self.docker_status.insert(host.id.clone(), status);
        }

        // SSH and Coolify would need async network checks
        // For now, keep them as Unknown
    }

    fn count_by_status(&self, statuses: &HashMap<String, ConnectionStatus>) -> (usize, usize, usize) {
        let online = statuses.values().filter(|s| **s == ConnectionStatus::Online).count();
        let offline = statuses.values().filter(|s| **s == ConnectionStatus::Offline).count();
        let unknown = statuses.values().filter(|s| **s == ConnectionStatus::Unknown).count();
        (online, offline, unknown)
    }
}

pub async fn run(config: Arc<Config>, db: Arc<Database>) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config, db);
    app.check_all_connections(); // Initial status check
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.size());

            // ─────────────────────────────────────────────────────────────────
            // Header
            // ─────────────────────────────────────────────────────────────────
            let header = Paragraph::new(Line::from(vec![
                Span::styled(
                    " pctrl ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    "Mission Control for Self-Hosters & Indie Devs",
                    Style::default().fg(Color::White),
                ),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            f.render_widget(header, chunks[0]);

            // ─────────────────────────────────────────────────────────────────
            // Main content
            // ─────────────────────────────────────────────────────────────────
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
                .split(chunks[1]);

            // Sidebar Menu
            let total_count = app.config.ssh_connections.len()
                + app.config.docker_hosts.len()
                + app.config.coolify_instances.len()
                + app.config.git_repos.len();

            let menu_items: Vec<ListItem> = [
                ("Status", total_count, SelectedPanel::Status),
                ("SSH", app.config.ssh_connections.len(), SelectedPanel::Ssh),
                (
                    "Docker",
                    app.config.docker_hosts.len(),
                    SelectedPanel::Docker,
                ),
                (
                    "Coolify",
                    app.config.coolify_instances.len(),
                    SelectedPanel::Coolify,
                ),
                ("Git", app.config.git_repos.len(), SelectedPanel::Git),
            ]
            .iter()
            .map(|(name, count, panel)| {
                let is_selected = app.selected_panel == *panel;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let prefix = if is_selected { "▶ " } else { "  " };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled((*name).to_string(), style),
                    Span::styled(
                        format!(" ({})", count),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect();

            let menu = List::new(menu_items).block(
                Block::default()
                    .title(" Menu ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            f.render_widget(menu, main_chunks[0]);

            // Content area - form or data
            let content = if app.input_mode == InputMode::Adding {
                // Render input form
                let fields = app.get_form_fields();
                let mut items: Vec<Line> = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("  Add New {}", match app.selected_panel {
                            SelectedPanel::Ssh => "SSH Connection",
                            SelectedPanel::Docker => "Docker Host",
                            SelectedPanel::Coolify => "Coolify Instance",
                            SelectedPanel::Git => "Git Repository",
                            SelectedPanel::Status => "",
                        }),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                ];

                for (i, (label, value)) in fields.iter().enumerate() {
                    let is_active = i == app.input_form.current_field;
                    let label_style = if is_active {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    let value_style = if is_active {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    let cursor = if is_active { "▌" } else { "" };
                    let prefix = if is_active { "▶ " } else { "  " };

                    items.push(Line::from(vec![
                        Span::styled(prefix, label_style),
                        Span::styled(format!("{:12}", label), label_style),
                        Span::styled(format!("{}{}", value, cursor), value_style),
                    ]));
                }

                if let Some(ref msg) = app.input_form.message {
                    items.push(Line::from(""));
                    items.push(Line::from(Span::styled(
                        format!("  {}", msg),
                        Style::default().fg(Color::Red),
                    )));
                }

                Paragraph::new(items)
            } else {
                // Normal content display
                match app.selected_panel {
                    SelectedPanel::Status => {
                    let (ssh_online, ssh_offline, ssh_unknown) = app.count_by_status(&app.ssh_status);
                    let (docker_online, docker_offline, docker_unknown) = app.count_by_status(&app.docker_status);
                    let (coolify_online, coolify_offline, coolify_unknown) = app.count_by_status(&app.coolify_status);
                    let (git_online, git_offline, git_unknown) = app.count_by_status(&app.git_status);

                    let ssh_total = ssh_online + ssh_offline + ssh_unknown;
                    let docker_total = docker_online + docker_offline + docker_unknown;
                    let coolify_total = coolify_online + coolify_offline + coolify_unknown;
                    let git_total = git_online + git_offline + git_unknown;

                    let mut items: Vec<Line> = vec![
                        Line::from(""),
                        Line::from(Span::styled(
                            "  Overview",
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Line::from(Span::styled(
                            "  Press 'r' to refresh status",
                            Style::default().fg(Color::DarkGray),
                        )),
                        Line::from(""),
                    ];

                    // Helper to build status line
                    fn build_status_line(name: &str, padding: &str, online: usize, offline: usize, unknown: usize) -> Line<'static> {
                        let total = online + offline + unknown;
                        if total == 0 {
                            return Line::from(vec![
                                Span::styled("  ○ ", Style::default().fg(Color::DarkGray)),
                                Span::styled(name.to_string(), Style::default().fg(Color::White)),
                                Span::styled(format!("{}{}", padding, 0), Style::default().fg(Color::DarkGray)),
                            ]);
                        }

                        let mut spans = vec![Span::raw("  ")];

                        // Show icons for each status
                        if online > 0 {
                            spans.push(Span::styled("●", Style::default().fg(Color::Green)));
                        }
                        if offline > 0 {
                            spans.push(Span::styled("●", Style::default().fg(Color::Red)));
                        }
                        if unknown > 0 && online == 0 && offline == 0 {
                            spans.push(Span::styled("●", Style::default().fg(Color::Yellow)));
                        }
                        spans.push(Span::raw(" "));

                        spans.push(Span::styled(name.to_string(), Style::default().fg(Color::White)));

                        // Build count display
                        let mut count_parts = Vec::new();
                        if online > 0 {
                            count_parts.push(format!("{} online", online));
                        }
                        if offline > 0 {
                            count_parts.push(format!("{} offline", offline));
                        }
                        if unknown > 0 {
                            count_parts.push(format!("{} ?", unknown));
                        }

                        spans.push(Span::styled(
                            format!("{}  ", padding.chars().take(padding.len().saturating_sub(count_parts.join(", ").len())).collect::<String>()),
                            Style::default().fg(Color::DarkGray),
                        ));

                        if online > 0 {
                            spans.push(Span::styled(format!("{}", online), Style::default().fg(Color::Green)));
                            if offline > 0 || unknown > 0 {
                                spans.push(Span::styled("/", Style::default().fg(Color::DarkGray)));
                            }
                        }
                        if offline > 0 {
                            spans.push(Span::styled(format!("{}", offline), Style::default().fg(Color::Red)));
                            if unknown > 0 {
                                spans.push(Span::styled("/", Style::default().fg(Color::DarkGray)));
                            }
                        }
                        if unknown > 0 {
                            spans.push(Span::styled(format!("{}", unknown), Style::default().fg(Color::Yellow)));
                        }

                        Line::from(spans)
                    }

                    items.push(build_status_line("SSH Connections", "      ", ssh_online, ssh_offline, ssh_unknown));
                    items.push(build_status_line("Docker Hosts", "         ", docker_online, docker_offline, docker_unknown));
                    items.push(build_status_line("Coolify Instances", "    ", coolify_online, coolify_offline, coolify_unknown));
                    items.push(build_status_line("Git Repositories", "     ", git_online, git_offline, git_unknown));

                    items.push(Line::from(""));
                    items.push(Line::from(Span::styled(
                        "  ─────────────────────────────",
                        Style::default().fg(Color::DarkGray),
                    )));

                    let total = ssh_total + docker_total + coolify_total + git_total;
                    let total_online = ssh_online + docker_online + coolify_online + git_online;
                    let total_offline = ssh_offline + docker_offline + coolify_offline + git_offline;

                    if total == 0 {
                        items.push(Line::from(""));
                        items.push(Line::from(Span::styled(
                            "  No resources configured yet.",
                            Style::default().fg(Color::DarkGray),
                        )));
                        items.push(Line::from(Span::styled(
                            "  Use ↓ to navigate and add resources.",
                            Style::default().fg(Color::Yellow),
                        )));
                    } else {
                        items.push(Line::from(""));
                        // Legend
                        items.push(Line::from(vec![
                            Span::styled("  ● ", Style::default().fg(Color::Green)),
                            Span::styled("Online", Style::default().fg(Color::DarkGray)),
                            Span::styled("  ● ", Style::default().fg(Color::Red)),
                            Span::styled("Offline", Style::default().fg(Color::DarkGray)),
                            Span::styled("  ● ", Style::default().fg(Color::Yellow)),
                            Span::styled("Unknown", Style::default().fg(Color::DarkGray)),
                        ]));
                        items.push(Line::from(""));
                        items.push(Line::from(vec![
                            Span::styled("  Total: ", Style::default().fg(Color::DarkGray)),
                            Span::styled(format!("{}", total_online), Style::default().fg(Color::Green)),
                            Span::styled(" online, ", Style::default().fg(Color::DarkGray)),
                            Span::styled(format!("{}", total_offline), Style::default().fg(Color::Red)),
                            Span::styled(" offline", Style::default().fg(Color::DarkGray)),
                        ]));
                    }

                    Paragraph::new(items)
                }
                SelectedPanel::Ssh => {
                    let items: Vec<Line> = if app.config.ssh_connections.is_empty() {
                        vec![
                            Line::from(""),
                            Line::from(Span::styled(
                                "  No SSH connections configured",
                                Style::default().fg(Color::DarkGray),
                            )),
                            Line::from(""),
                            Line::from(Span::styled(
                                "  Add with: pctrl ssh add <name> <host> <user>",
                                Style::default().fg(Color::Yellow),
                            )),
                        ]
                    } else {
                        app.config
                            .ssh_connections
                            .iter()
                            .map(|conn| {
                                Line::from(vec![
                                    Span::styled("  ● ", Style::default().fg(Color::Green)),
                                    Span::styled(&conn.name, Style::default().fg(Color::Cyan)),
                                    Span::raw(" - "),
                                    Span::styled(
                                        format!("{}@{}:{}", conn.username, conn.host, conn.port),
                                        Style::default().fg(Color::White),
                                    ),
                                ])
                            })
                            .collect()
                    };
                    Paragraph::new(items)
                }
                SelectedPanel::Docker => {
                    let items: Vec<Line> = if app.config.docker_hosts.is_empty() {
                        vec![
                            Line::from(""),
                            Line::from(Span::styled(
                                "  No Docker hosts configured",
                                Style::default().fg(Color::DarkGray),
                            )),
                            Line::from(""),
                            Line::from(Span::styled(
                                "  Add with: pctrl docker add <name> <url>",
                                Style::default().fg(Color::Yellow),
                            )),
                        ]
                    } else {
                        app.config
                            .docker_hosts
                            .iter()
                            .map(|host| {
                                Line::from(vec![
                                    Span::styled("  ● ", Style::default().fg(Color::Blue)),
                                    Span::styled(&host.name, Style::default().fg(Color::Cyan)),
                                    Span::raw(" - "),
                                    Span::styled(&host.url, Style::default().fg(Color::White)),
                                ])
                            })
                            .collect()
                    };
                    Paragraph::new(items)
                }
                SelectedPanel::Coolify => {
                    let items: Vec<Line> = if app.config.coolify_instances.is_empty() {
                        vec![
                            Line::from(""),
                            Line::from(Span::styled(
                                "  No Coolify instances configured",
                                Style::default().fg(Color::DarkGray),
                            )),
                            Line::from(""),
                            Line::from(Span::styled(
                                "  Add with: pctrl coolify add <name> <url> <token>",
                                Style::default().fg(Color::Yellow),
                            )),
                        ]
                    } else {
                        app.config
                            .coolify_instances
                            .iter()
                            .map(|instance| {
                                Line::from(vec![
                                    Span::styled("  ● ", Style::default().fg(Color::Magenta)),
                                    Span::styled(&instance.name, Style::default().fg(Color::Cyan)),
                                    Span::raw(" - "),
                                    Span::styled(&instance.url, Style::default().fg(Color::White)),
                                ])
                            })
                            .collect()
                    };
                    Paragraph::new(items)
                }
                SelectedPanel::Git => {
                    let items: Vec<Line> = if app.config.git_repos.is_empty() {
                        vec![
                            Line::from(""),
                            Line::from(Span::styled(
                                "  No Git repositories configured",
                                Style::default().fg(Color::DarkGray),
                            )),
                            Line::from(""),
                            Line::from(Span::styled(
                                "  Add with: pctrl git add <name> <path>",
                                Style::default().fg(Color::Yellow),
                            )),
                        ]
                    } else {
                        app.config
                            .git_repos
                            .iter()
                            .map(|repo| {
                                Line::from(vec![
                                    Span::styled("  ● ", Style::default().fg(Color::Yellow)),
                                    Span::styled(&repo.name, Style::default().fg(Color::Cyan)),
                                    Span::raw(" - "),
                                    Span::styled(&repo.path, Style::default().fg(Color::White)),
                                ])
                            })
                            .collect()
                    };
                    Paragraph::new(items)
                }
            }
            }  // close the else block
            .block(
                Block::default()
                    .title(format!(
                        " {} ",
                        match app.selected_panel {
                            SelectedPanel::Status => "Status",
                            SelectedPanel::Ssh => "SSH Connections",
                            SelectedPanel::Docker => "Docker Hosts",
                            SelectedPanel::Coolify => "Coolify Instances",
                            SelectedPanel::Git => "Git Repositories",
                        }
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            f.render_widget(content, main_chunks[1]);

            // ─────────────────────────────────────────────────────────────────
            // Footer with navigation hints and status
            // ─────────────────────────────────────────────────────────────────
            let footer_content = if app.input_mode == InputMode::Adding {
                // Form mode footer
                Line::from(vec![
                    Span::styled(" Tab ", Style::default().fg(Color::Cyan)),
                    Span::raw("Next"),
                    Span::raw("  │  "),
                    Span::styled(" Shift+Tab ", Style::default().fg(Color::Cyan)),
                    Span::raw("Prev"),
                    Span::raw("  │  "),
                    Span::styled(" Enter ", Style::default().fg(Color::Cyan)),
                    Span::raw("Save"),
                    Span::raw("  │  "),
                    Span::styled(" Esc ", Style::default().fg(Color::Cyan)),
                    Span::raw("Cancel"),
                ])
            } else {
                // Normal mode footer
                let can_add = app.selected_panel != SelectedPanel::Status;
                let mut spans = vec![
                    Span::styled(" ↑↓ ", Style::default().fg(Color::Cyan)),
                    Span::raw("Navigate"),
                ];
                if can_add {
                    spans.extend(vec![
                        Span::raw("  │  "),
                        Span::styled(" a ", Style::default().fg(Color::Cyan)),
                        Span::raw("Add"),
                    ]);
                }
                spans.extend(vec![
                    Span::raw("  │  "),
                    Span::styled(" r ", Style::default().fg(Color::Cyan)),
                    Span::raw("Refresh"),
                    Span::raw("  │  "),
                    Span::styled(" q ", Style::default().fg(Color::Cyan)),
                    Span::raw("Quit"),
                ]);
                Line::from(spans)
            };

            let footer = Paragraph::new(footer_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            f.render_widget(footer, chunks[2]);
        })?;

        // Handle input - nur auf Press reagieren (nicht Release)
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Ignoriere Release-Events (Windows sendet Press + Release)
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match app.input_mode {
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.selected_panel = match app.selected_panel {
                                    SelectedPanel::Status => SelectedPanel::Ssh,
                                    SelectedPanel::Ssh => SelectedPanel::Docker,
                                    SelectedPanel::Docker => SelectedPanel::Coolify,
                                    SelectedPanel::Coolify => SelectedPanel::Git,
                                    SelectedPanel::Git => SelectedPanel::Status,
                                };
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.selected_panel = match app.selected_panel {
                                    SelectedPanel::Status => SelectedPanel::Git,
                                    SelectedPanel::Ssh => SelectedPanel::Status,
                                    SelectedPanel::Docker => SelectedPanel::Ssh,
                                    SelectedPanel::Coolify => SelectedPanel::Docker,
                                    SelectedPanel::Git => SelectedPanel::Coolify,
                                };
                            }
                            KeyCode::Char('a') => {
                                // Start adding new entry (not on Status panel)
                                if app.selected_panel != SelectedPanel::Status {
                                    app.reset_form();
                                    app.input_mode = InputMode::Adding;
                                }
                            }
                            KeyCode::Char('r') => {
                                app.check_all_connections();
                            }
                            _ => {}
                        }
                    }
                    InputMode::Adding => {
                        match key.code {
                            KeyCode::Esc => {
                                // Cancel adding
                                app.input_mode = InputMode::Normal;
                                app.reset_form();
                            }
                            KeyCode::Tab => {
                                // Next field
                                let count = app.field_count();
                                if count > 0 {
                                    app.input_form.current_field = (app.input_form.current_field + 1) % count;
                                }
                            }
                            KeyCode::BackTab => {
                                // Previous field
                                let count = app.field_count();
                                if count > 0 {
                                    app.input_form.current_field = if app.input_form.current_field == 0 {
                                        count - 1
                                    } else {
                                        app.input_form.current_field - 1
                                    };
                                }
                            }
                            KeyCode::Enter => {
                                // Save the new entry
                                if let Err(e) = save_new_entry(app).await {
                                    app.input_form.message = Some(format!("Error: {}", e));
                                } else {
                                    app.input_mode = InputMode::Normal;
                                    app.reset_form();
                                    app.check_all_connections();
                                }
                            }
                            KeyCode::Backspace => {
                                if let Some(input) = app.current_input_mut() {
                                    input.pop();
                                }
                            }
                            KeyCode::Char(c) => {
                                if let Some(input) = app.current_input_mut() {
                                    input.push(c);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

/// Save a new entry based on current panel and form data
async fn save_new_entry(app: &mut App) -> anyhow::Result<()> {
    let id = Uuid::new_v4().to_string();

    // Get mutable config
    let config = Arc::make_mut(&mut app.config);

    match app.selected_panel {
        SelectedPanel::Ssh => {
            if app.input_form.name.is_empty() || app.input_form.host.is_empty() {
                anyhow::bail!("Name and Host are required");
            }
            let port: u16 = app.input_form.port.parse().unwrap_or(22);
            let username = if app.input_form.user.is_empty() {
                "root".to_string()
            } else {
                app.input_form.user.clone()
            };

            let conn = SshConnection {
                id: id.clone(),
                name: app.input_form.name.clone(),
                host: app.input_form.host.clone(),
                port,
                username,
                auth_method: AuthMethod::PublicKey {
                    key_path: "~/.ssh/id_rsa".to_string(),
                },
            };
            config.ssh_connections.push(conn);
            app.ssh_status.insert(id, ConnectionStatus::Unknown);
        }
        SelectedPanel::Docker => {
            if app.input_form.name.is_empty() {
                anyhow::bail!("Name is required");
            }
            let url = if app.input_form.url.is_empty() {
                "unix:///var/run/docker.sock".to_string()
            } else {
                app.input_form.url.clone()
            };

            let host = DockerHost {
                id: id.clone(),
                name: app.input_form.name.clone(),
                url,
            };
            config.docker_hosts.push(host);
            app.docker_status.insert(id, ConnectionStatus::Unknown);
        }
        SelectedPanel::Coolify => {
            if app.input_form.name.is_empty() || app.input_form.url.is_empty() || app.input_form.token.is_empty() {
                anyhow::bail!("Name, URL, and Token are required");
            }

            let instance = CoolifyInstance {
                id: id.clone(),
                name: app.input_form.name.clone(),
                url: app.input_form.url.clone(),
                api_key: app.input_form.token.clone(),
            };
            config.coolify_instances.push(instance);
            app.coolify_status.insert(id, ConnectionStatus::Unknown);
        }
        SelectedPanel::Git => {
            if app.input_form.name.is_empty() || app.input_form.path.is_empty() {
                anyhow::bail!("Name and Path are required");
            }

            let repo = GitRepo {
                id: id.clone(),
                name: app.input_form.name.clone(),
                path: app.input_form.path.clone(),
                remote_url: None,
            };
            config.git_repos.push(repo);
            app.git_status.insert(id, ConnectionStatus::Unknown);
        }
        SelectedPanel::Status => {}
    }

    // Save config to database
    app.db.save_config(config).await?;

    Ok(())
}
