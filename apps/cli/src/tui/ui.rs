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
        + app.servers.len()
        + app.domains.len()
        + app.databases.len()
        + app.scripts.len();

    let menu_items: Vec<ListItem> = [
        ("Status", total_count, SelectedPanel::Status),
        ("Projects", app.projects.len(), SelectedPanel::Projects),
        ("Servers", app.servers.len(), SelectedPanel::Servers),
        ("Domains", app.domains.len(), SelectedPanel::Domains),
        ("Databases", app.databases.len(), SelectedPanel::Databases),
        ("Scripts", app.scripts.len(), SelectedPanel::Scripts),
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
            SelectedPanel::Servers => render_servers(app),
            SelectedPanel::Domains => render_domains(app),
            SelectedPanel::Databases => render_databases(app),
            SelectedPanel::Scripts => render_scripts(app),
        }
    }
    .block(
        Block::default()
            .title(format!(
                " {} ",
                match app.selected_panel {
                    SelectedPanel::Status => "Status",
                    SelectedPanel::Projects => "Projects",
                    SelectedPanel::Servers => "Servers",
                    SelectedPanel::Domains => "Domains",
                    SelectedPanel::Databases => "Databases",
                    SelectedPanel::Scripts => "Scripts",
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
                    SelectedPanel::Servers => "Server",
                    SelectedPanel::Domains => "Domain",
                    SelectedPanel::Databases => "Database",
                    SelectedPanel::Scripts => "Script",
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
    let mut items: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Overview",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Resource counts
    items.push(Line::from(vec![
        Span::styled("  ● ", Style::default().fg(Color::Cyan)),
        Span::styled("Projects", Style::default().fg(Color::White)),
        Span::styled(
            format!("         {}", app.projects.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    items.push(Line::from(vec![
        Span::styled("  ● ", Style::default().fg(Color::Green)),
        Span::styled("Servers", Style::default().fg(Color::White)),
        Span::styled(
            format!("          {}", app.servers.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    items.push(Line::from(vec![
        Span::styled("  ● ", Style::default().fg(Color::Blue)),
        Span::styled("Domains", Style::default().fg(Color::White)),
        Span::styled(
            format!("          {}", app.domains.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    items.push(Line::from(vec![
        Span::styled("  ● ", Style::default().fg(Color::Magenta)),
        Span::styled("Databases", Style::default().fg(Color::White)),
        Span::styled(
            format!("        {}", app.databases.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    items.push(Line::from(vec![
        Span::styled("  ● ", Style::default().fg(Color::Yellow)),
        Span::styled("Scripts", Style::default().fg(Color::White)),
        Span::styled(
            format!("          {}", app.scripts.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    let total = app.projects.len()
        + app.servers.len()
        + app.domains.len()
        + app.databases.len()
        + app.scripts.len();

    items.push(Line::from(""));
    items.push(Line::from(Span::styled(
        "  ─────────────────────────────",
        Style::default().fg(Color::DarkGray),
    )));

    if total == 0 {
        items.push(Line::from(""));
        items.push(Line::from(Span::styled(
            "  No resources configured yet.",
            Style::default().fg(Color::DarkGray),
        )));
        items.push(Line::from(Span::styled(
            "  Use ↓ to navigate and 'a' to add resources.",
            Style::default().fg(Color::Yellow),
        )));
    } else {
        items.push(Line::from(""));
        items.push(Line::from(vec![
            Span::styled("  Total: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", total), Style::default().fg(Color::Cyan)),
            Span::styled(" resources", Style::default().fg(Color::DarkGray)),
        ]));
    }

    // Legacy migration warning
    let legacy_count = app.total_legacy_count();
    if legacy_count > 0 {
        items.push(Line::from(""));
        items.push(Line::from(Span::styled(
            "  ⚠ Legacy Data Warning",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        items.push(Line::from(Span::styled(
            format!("  {} legacy entries found.", legacy_count),
            Style::default().fg(Color::Yellow),
        )));
        items.push(Line::from(Span::styled(
            "  Run: pctrl migrate",
            Style::default().fg(Color::Yellow),
        )));
    }

    Paragraph::new(items)
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

fn render_servers(app: &App) -> Paragraph<'static> {
    let items: Vec<Line> = if app.servers.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No servers configured",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press 'a' to add a server, or use:",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "  pctrl server add <name> <host>",
                Style::default().fg(Color::Yellow),
            )),
        ]
    } else {
        app.servers
            .iter()
            .map(|server| {
                let type_str = format!(" [{}]", server.server_type);
                Line::from(vec![
                    Span::styled("  ● ", Style::default().fg(Color::Green)),
                    Span::styled(server.name.clone(), Style::default().fg(Color::Cyan)),
                    Span::raw(" - "),
                    Span::styled(server.host.clone(), Style::default().fg(Color::White)),
                    Span::styled(type_str, Style::default().fg(Color::DarkGray)),
                ])
            })
            .collect()
    };
    Paragraph::new(items)
}

fn render_domains(app: &App) -> Paragraph<'static> {
    let items: Vec<Line> = if app.domains.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No domains configured",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press 'a' to add a domain, or use:",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "  pctrl domain add <domain>",
                Style::default().fg(Color::Yellow),
            )),
        ]
    } else {
        app.domains
            .iter()
            .map(|domain| {
                let ssl_icon = if domain.ssl { "🔒" } else { "🔓" };
                let type_str = format!(" ({})", domain.domain_type);
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::raw(ssl_icon),
                    Span::raw(" "),
                    Span::styled(domain.domain.clone(), Style::default().fg(Color::Blue)),
                    Span::styled(type_str, Style::default().fg(Color::DarkGray)),
                ])
            })
            .collect()
    };
    Paragraph::new(items)
}

fn render_databases(app: &App) -> Paragraph<'static> {
    let items: Vec<Line> = if app.databases.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No databases configured",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press 'a' to add a database, or use:",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "  pctrl db add <name> <type> <host>",
                Style::default().fg(Color::Yellow),
            )),
        ]
    } else {
        app.databases
            .iter()
            .map(|db| {
                let host_str = db.host.as_deref().unwrap_or("localhost");
                let port_str = db.port.map(|p| format!(":{}", p)).unwrap_or_default();
                Line::from(vec![
                    Span::styled("  ● ", Style::default().fg(Color::Magenta)),
                    Span::styled(db.name.clone(), Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!(" [{}]", db.db_type),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::raw(" - "),
                    Span::styled(
                        format!("{}{}", host_str, port_str),
                        Style::default().fg(Color::White),
                    ),
                ])
            })
            .collect()
    };
    Paragraph::new(items)
}

fn render_scripts(app: &App) -> Paragraph<'static> {
    let items: Vec<Line> = if app.scripts.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No scripts configured",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press 'a' to add a script, or use:",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "  pctrl script add <name> <command>",
                Style::default().fg(Color::Yellow),
            )),
        ]
    } else {
        app.scripts
            .iter()
            .map(|script| {
                let type_str = format!(" [{}]", script.script_type);
                let cmd_preview: String = script.command.chars().take(40).collect();
                let cmd_display = if script.command.len() > 40 {
                    format!("{}...", cmd_preview)
                } else {
                    cmd_preview
                };
                Line::from(vec![
                    Span::styled("  ● ", Style::default().fg(Color::Yellow)),
                    Span::styled(script.name.clone(), Style::default().fg(Color::Cyan)),
                    Span::styled(type_str, Style::default().fg(Color::Yellow)),
                    Span::raw(" - "),
                    Span::styled(cmd_display, Style::default().fg(Color::DarkGray)),
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
