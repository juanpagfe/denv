//! TUI mode using ratatui for interactive network monitoring.

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Terminal,
};

use crate::config::Config;
use crate::tracker::{Connection, Tracker};

pub fn run(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, config);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tracker = Tracker::new();
    let mut table_state = TableState::default();
    let mut connections: Vec<Connection> = Vec::new();
    let interval = Duration::from_millis(config.interval_ms);
    let mut last_refresh = Instant::now() - interval; // Force immediate first refresh

    loop {
        // Refresh data if interval elapsed
        if last_refresh.elapsed() >= interval {
            connections = tracker.refresh(config);
            last_refresh = Instant::now();
        }

        // Draw
        terminal.draw(|f| {
            let chunks = Layout::vertical([
                Constraint::Length(1),  // Header bar
                Constraint::Min(5),    // Connection table
                Constraint::Length(1),  // Footer
            ])
            .split(f.area());

            draw_header(f, chunks[0], &connections);
            draw_table(f, chunks[1], &connections, &mut table_state);
            draw_footer(f, chunks[2]);
        })?;

        // Handle input (non-blocking with timeout)
        let poll_time = interval
            .checked_sub(last_refresh.elapsed())
            .unwrap_or(Duration::from_millis(50));

        if event::poll(poll_time)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = table_state.selected().unwrap_or(0);
                        if i < connections.len().saturating_sub(1) {
                            table_state.select(Some(i + 1));
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let i = table_state.selected().unwrap_or(0);
                        if i > 0 {
                            table_state.select(Some(i - 1));
                        }
                    }
                    KeyCode::Home | KeyCode::Char('g') => {
                        table_state.select(Some(0));
                    }
                    KeyCode::End | KeyCode::Char('G') => {
                        if !connections.is_empty() {
                            table_state.select(Some(connections.len() - 1));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn draw_header(f: &mut ratatui::Frame, area: Rect, connections: &[Connection]) {
    let established = connections.iter().filter(|c| c.state.is_established()).count();
    let listening = connections.iter().filter(|c| c.state.is_listening()).count();
    let total = connections.len();

    let header = Line::from(vec![
        Span::styled(" netwatch ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("│ "),
        Span::styled(format!("{total}"), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" connections  "),
        Span::styled(format!("{established}"), Style::default().fg(Color::Green)),
        Span::raw(" established  "),
        Span::styled(format!("{listening}"), Style::default().fg(Color::Cyan)),
        Span::raw(" listening"),
    ]);

    f.render_widget(Paragraph::new(header), area);
}

fn draw_table(
    f: &mut ratatui::Frame,
    area: Rect,
    connections: &[Connection],
    state: &mut TableState,
) {
    let header = Row::new([
        Cell::from("PID"),
        Cell::from("PROCESS"),
        Cell::from("PROTO"),
        Cell::from("STATE"),
        Cell::from("LOCAL"),
        Cell::from("REMOTE"),
        Cell::from("USER"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD))
    .height(1);

    let rows: Vec<Row> = connections
        .iter()
        .map(|conn| {
            let pid_str = conn
                .pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string());

            let remote_str = match &conn.hostname {
                Some(host) => format!("{}:{}", host, conn.remote.port()),
                None => conn.remote.to_string(),
            };

            let state_style = match conn.state.as_str() {
                "ESTABLISHED" => Style::default().fg(Color::Green),
                "LISTEN" => Style::default().fg(Color::Cyan),
                "TIME_WAIT" | "CLOSE_WAIT" => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::DarkGray),
            };

            Row::new([
                Cell::from(pid_str),
                Cell::from(conn.process_name.clone()),
                Cell::from(conn.protocol.as_str()),
                Cell::from(conn.state.as_str()).style(state_style),
                Cell::from(conn.local.to_string()),
                Cell::from(remote_str),
                Cell::from(conn.user.clone()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(7),
        Constraint::Length(16),
        Constraint::Length(5),
        Constraint::Length(12),
        Constraint::Length(22),
        Constraint::Min(22),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .row_highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED),
        );

    f.render_stateful_widget(table, area, state);
}

fn draw_footer(f: &mut ratatui::Frame, area: Rect) {
    let footer = Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Yellow)),
        Span::raw(" quit  "),
        Span::styled("↑↓/jk", Style::default().fg(Color::Yellow)),
        Span::raw(" navigate  "),
    ]);

    f.render_widget(
        Paragraph::new(footer).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}
