//! TUI UI rendering

use super::app::App;
use super::types::{InputMode, SelectedPanel};
use pctrl_core::ProjectStatus;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    render_header(f, chunks[0]);
    render_main(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect) {
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
    f.render_widget(header, area);
}

fn render_main(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    render_sidebar(f, app, main_chunks[0]);
    render_content(f, app, main_chunks[1]);
}

fn render_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let total_count = app.projects.len()
        + app.config.ssh_connections.len()
        + app.config.docker_hosts.len()
        + app.config.coolify_instances.len()
        + app.config.git_repos.len();

    let menu_items: Vec<ListItem> = [
        ("Status", total_count, SelectedPanel::Status),
        ("Projects", app.projects.len(), SelectedPanel::Projects),
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
    f.render_widget(menu, area);
}

fn render_content(f: &mut Frame, app: &App, area: Rect) {
    let content = if app.input_mode == InputMode::Adding {
        render_form(app)
    } else {
        match app.selected_panel {
            SelectedPanel::Status => render_status(app),
            SelectedPanel::Projects => render_projects(app),
            SelectedPanel::Ssh => render_ssh(app),
            SelectedPanel::Docker => render_docker(app),
            SelectedPanel::Coolify => render_coolify(app),
            SelectedPanel::Git => render_git(app),
        }
    }
    .block(
        Block::default()
            .title(format!(
                " {} ",
                match app.selected_panel {
                    SelectedPanel::Status => "Status",
                    SelectedPanel::Projects => "Projects",
                    SelectedPanel::Ssh => "SSH Connections",
                    SelectedPanel::Docker => "Docker Hosts",
                    SelectedPanel::Coolify => "Coolify Instances",
                    SelectedPanel::Git => "Git Repositories",
                }
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(content, area);
}

fn render_form(app: &App) -> Paragraph<'static> {
    let fields = app.get_form_fields();
    let mut items: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "  Add New {}",
                match app.selected_panel {
                    SelectedPanel::Projects => "Project",
                    SelectedPanel::Ssh => "SSH Connection",
                    SelectedPanel::Docker => "Docker Host",
                    SelectedPanel::Coolify => "Coolify Instance",
                    SelectedPanel::Git => "Git Repository",
                    SelectedPanel::Status => "",
                }
            ),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (i, (label, value)) in fields.iter().enumerate() {
        let is_active = i == app.input_form.current_field;
        let label_style = if is_active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let value_style = if is_active {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let cursor = if is_active { "▌" } else { "" };
        let prefix = if is_active { "▶ " } else { "  " };

        items.push(Line::from(vec![
            Span::styled(prefix.to_string(), label_style),
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
}

fn render_status(app: &App) -> Paragraph<'static> {
    let (ssh_online, ssh_offline, ssh_unknown) = app.count_by_status(&app.ssh_status);
    let (docker_online, docker_offline, docker_unknown) = app.count_by_status(&app.docker_status);
    let (coolify_online, coolify_offline, coolify_unknown) =
        app.count_by_status(&app.coolify_status);
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

    items.push(build_status_line(
        "SSH Connections",
        "      ",
        ssh_online,
        ssh_offline,
        ssh_unknown,
    ));
    items.push(build_status_line(
        "Docker Hosts",
        "         ",
        docker_online,
        docker_offline,
        docker_unknown,
    ));
    items.push(build_status_line(
        "Coolify Instances",
        "    ",
        coolify_online,
        coolify_offline,
        coolify_unknown,
    ));
    items.push(build_status_line(
        "Git Repositories",
        "     ",
        git_online,
        git_offline,
        git_unknown,
    ));

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
            Span::styled(
                format!("{}", total_online),
                Style::default().fg(Color::Green),
            ),
            Span::styled(" online, ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", total_offline),
                Style::default().fg(Color::Red),
            ),
            Span::styled(" offline", Style::default().fg(Color::DarkGray)),
        ]));
    }

    Paragraph::new(items)
}

fn build_status_line(
    name: &str,
    padding: &str,
    online: usize,
    offline: usize,
    unknown: usize,
) -> Line<'static> {
    let total = online + offline + unknown;
    if total == 0 {
        return Line::from(vec![
            Span::styled("  ○ ", Style::default().fg(Color::DarkGray)),
            Span::styled(name.to_string(), Style::default().fg(Color::White)),
            Span::styled(
                format!("{}{}", padding, 0),
                Style::default().fg(Color::DarkGray),
            ),
        ]);
    }

    let mut spans = vec![Span::raw("  ")];

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

    spans.push(Span::styled(
        name.to_string(),
        Style::default().fg(Color::White),
    ));

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
        format!(
            "{}  ",
            padding
                .chars()
                .take(padding.len().saturating_sub(count_parts.join(", ").len()))
                .collect::<String>()
        ),
        Style::default().fg(Color::DarkGray),
    ));

    if online > 0 {
        spans.push(Span::styled(
            format!("{}", online),
            Style::default().fg(Color::Green),
        ));
        if offline > 0 || unknown > 0 {
            spans.push(Span::styled("/", Style::default().fg(Color::DarkGray)));
        }
    }
    if offline > 0 {
        spans.push(Span::styled(
            format!("{}", offline),
            Style::default().fg(Color::Red),
        ));
        if unknown > 0 {
            spans.push(Span::styled("/", Style::default().fg(Color::DarkGray)));
        }
    }
    if unknown > 0 {
        spans.push(Span::styled(
            format!("{}", unknown),
            Style::default().fg(Color::Yellow),
        ));
    }

    Line::from(spans)
}

fn render_projects(app: &App) -> Paragraph<'static> {
    let items: Vec<Line> = if app.projects.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No projects configured",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press 'a' to add a project, or use:",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "  pctrl project add <name>",
                Style::default().fg(Color::Yellow),
            )),
        ]
    } else {
        app.projects
            .iter()
            .map(|project| {
                let status_color = match project.status {
                    ProjectStatus::Dev => Color::Yellow,
                    ProjectStatus::Staging => Color::Blue,
                    ProjectStatus::Live => Color::Green,
                    ProjectStatus::Archived => Color::DarkGray,
                };
                let stack_str = if project.stack.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", project.stack.join(", "))
                };
                Line::from(vec![
                    Span::styled("  ● ", Style::default().fg(status_color)),
                    Span::styled(project.name.clone(), Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!(" ({})", project.status),
                        Style::default().fg(status_color),
                    ),
                    Span::styled(stack_str, Style::default().fg(Color::DarkGray)),
                ])
            })
            .collect()
    };
    Paragraph::new(items)
}

fn render_ssh(app: &App) -> Paragraph<'static> {
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
                    Span::styled(conn.name.clone(), Style::default().fg(Color::Cyan)),
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

fn render_docker(app: &App) -> Paragraph<'static> {
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
                    Span::styled(host.name.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw(" - "),
                    Span::styled(host.url.clone(), Style::default().fg(Color::White)),
                ])
            })
            .collect()
    };
    Paragraph::new(items)
}

fn render_coolify(app: &App) -> Paragraph<'static> {
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
                    Span::styled(instance.name.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw(" - "),
                    Span::styled(instance.url.clone(), Style::default().fg(Color::White)),
                ])
            })
            .collect()
    };
    Paragraph::new(items)
}

fn render_git(app: &App) -> Paragraph<'static> {
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
                    Span::styled(repo.name.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw(" - "),
                    Span::styled(repo.path.clone(), Style::default().fg(Color::White)),
                ])
            })
            .collect()
    };
    Paragraph::new(items)
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let footer_content = if app.input_mode == InputMode::Adding {
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

    let footer = Paragraph::new(footer_content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(footer, area);
}
