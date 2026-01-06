use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

enum SelectedPanel {
    Ssh,
    Docker,
    Coolify,
    Git,
}

struct App {
    selected_panel: SelectedPanel,
}

impl App {
    fn new() -> Self {
        Self {
            selected_panel: SelectedPanel::Ssh,
        }
    }
}

pub async fn run() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
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
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.size());

            // Header
            let header = Paragraph::new("pctrl - Mission Control for Self-Hosters & Indie Devs")
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // Main content
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(chunks[1]);

            // Sidebar
            let items = vec![
                ListItem::new("SSH Connections"),
                ListItem::new("Docker Containers"),
                ListItem::new("Coolify Deployments"),
                ListItem::new("Git Releases"),
            ];
            let list = List::new(items)
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::DarkGray));
            f.render_widget(list, main_chunks[0]);

            // Content area
            let content = match app.selected_panel {
                SelectedPanel::Ssh => {
                    Paragraph::new("SSH Connections\n\nNo connections configured yet.")
                }
                SelectedPanel::Docker => {
                    Paragraph::new("Docker Containers\n\nNo Docker hosts configured yet.")
                }
                SelectedPanel::Coolify => {
                    Paragraph::new("Coolify Deployments\n\nNo Coolify instances configured yet.")
                }
                SelectedPanel::Git => {
                    Paragraph::new("Git Releases\n\nNo Git repositories configured yet.")
                }
            }
            .block(Block::default().title("Details").borders(Borders::ALL));
            f.render_widget(content, main_chunks[1]);

            // Footer
            let footer = Paragraph::new("Press 'q' to quit | Arrow keys to navigate")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => {
                        app.selected_panel = match app.selected_panel {
                            SelectedPanel::Ssh => SelectedPanel::Docker,
                            SelectedPanel::Docker => SelectedPanel::Coolify,
                            SelectedPanel::Coolify => SelectedPanel::Git,
                            SelectedPanel::Git => SelectedPanel::Ssh,
                        };
                    }
                    KeyCode::Up => {
                        app.selected_panel = match app.selected_panel {
                            SelectedPanel::Ssh => SelectedPanel::Git,
                            SelectedPanel::Docker => SelectedPanel::Ssh,
                            SelectedPanel::Coolify => SelectedPanel::Docker,
                            SelectedPanel::Git => SelectedPanel::Coolify,
                        };
                    }
                    _ => {}
                }
            }
        }
    }
}
