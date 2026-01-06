use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use pctrl_core::Config;
use pctrl_database::Database;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq)]
enum SelectedPanel {
    Ssh,
    Docker,
    Coolify,
    Git,
}

struct App {
    selected_panel: SelectedPanel,
    config: Arc<Config>,
    #[allow(dead_code)]
    db: Arc<Database>,
}

impl App {
    fn new(config: Arc<Config>, db: Arc<Database>) -> Self {
        Self {
            selected_panel: SelectedPanel::Ssh,
            config,
            db,
        }
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
            let menu_items: Vec<ListItem> = vec![
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
                    Span::styled(format!("{}", name), style),
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

            // Content area - zeigt echte Daten!
            let content = match app.selected_panel {
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
            .block(
                Block::default()
                    .title(format!(
                        " {} ",
                        match app.selected_panel {
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
            // Footer
            // ─────────────────────────────────────────────────────────────────
            let footer = Paragraph::new(Line::from(vec![
                Span::styled(" ↑↓ ", Style::default().fg(Color::Cyan)),
                Span::raw("Navigate"),
                Span::raw("  │  "),
                Span::styled(" Enter ", Style::default().fg(Color::Cyan)),
                Span::raw("Select"),
                Span::raw("  │  "),
                Span::styled(" q ", Style::default().fg(Color::Cyan)),
                Span::raw("Quit"),
            ]))
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

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.selected_panel = match app.selected_panel {
                            SelectedPanel::Ssh => SelectedPanel::Docker,
                            SelectedPanel::Docker => SelectedPanel::Coolify,
                            SelectedPanel::Coolify => SelectedPanel::Git,
                            SelectedPanel::Git => SelectedPanel::Ssh,
                        };
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.selected_panel = match app.selected_panel {
                            SelectedPanel::Ssh => SelectedPanel::Git,
                            SelectedPanel::Docker => SelectedPanel::Ssh,
                            SelectedPanel::Coolify => SelectedPanel::Docker,
                            SelectedPanel::Git => SelectedPanel::Coolify,
                        };
                    }
                    KeyCode::Enter => {
                        // TODO: Detail-Ansicht oder Aktionen für ausgewähltes Panel
                        // Für jetzt: nichts tun, später erweitern
                    }
                    _ => {}
                }
            }
        }
    }
}
