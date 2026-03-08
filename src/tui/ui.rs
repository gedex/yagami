use crate::tui::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(5),  // ASCII art title
            Constraint::Length(3),  // Pages progress bar
            Constraint::Length(8),  // Statistics
            Constraint::Length(6),  // Workers
            Constraint::Min(1),     // Spacer
            Constraint::Length(1),  // Help
        ])
        .split(f.area());

    // ASCII art title - centered
    let title_lines = vec![
        Line::from(" _   _                       _ "),
        Line::from("| | | |__ _ __ _ __ _ _ __ _(_)"),
        Line::from("| |_| / _` / _` / _` | '  \\| |"),
        Line::from(" \\__, \\__,_\\__, \\__,_|_|_|_|_|"),
        Line::from(" |___/     |___/               "),
    ];
    let title = Paragraph::new(title_lines)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Pages progress
    let pages_ratio = if app.stats.total_pages > 0 {
        app.stats.pages_crawled as f64 / app.stats.total_pages as f64
    } else {
        0.0
    };
    let pages_label = format!(
        "Pages: {}/{} ({:.0}%)",
        app.stats.pages_crawled,
        app.stats.total_pages,
        pages_ratio * 100.0
    );
    let pages_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .label(Span::styled(
            pages_label,
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ))
        .ratio(pages_ratio);
    f.render_widget(pages_gauge, chunks[1]);

    // Statistics
    let stats_text = vec![
        Line::from(vec![
            Span::styled("2xx Success:  ", Style::default().fg(Color::Green)),
            Span::raw(format!("{}", app.stats.get_2xx_count())),
        ]),
        Line::from(vec![
            Span::styled("3xx Redirect: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", app.stats.get_3xx_count())),
        ]),
        Line::from(vec![
            Span::styled("4xx Client:   ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}", app.stats.get_4xx_count())),
        ]),
        Line::from(vec![
            Span::styled("5xx Server:   ", Style::default().fg(Color::Magenta)),
            Span::raw(format!("{}", app.stats.get_5xx_count())),
        ]),
    ];
    let stats = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Statistics"));
    f.render_widget(stats, chunks[2]);

    // Workers section
    let active_pages = app.stats.total_pages.saturating_sub(app.stats.pages_crawled);

    let workers_text = vec![
        Line::from(vec![
            Span::styled("Page Workers:   ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} configured", app.page_workers)),
        ]),
        Line::from(vec![
            Span::styled("Link Checkers:  ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} configured", app.link_checkers)),
        ]),
        Line::from(vec![
            Span::styled("Links Checked:  ", Style::default().fg(Color::Green)),
            Span::raw(format!("{}", app.stats.links_checked)),
        ]),
        Line::from(vec![
            Span::styled("Pending Pages:  ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", active_pages)),
        ]),
    ];
    let workers = Paragraph::new(workers_text)
        .block(Block::default().borders(Borders::ALL).title("Workers"));
    f.render_widget(workers, chunks[3]);

    // Help text
    let help = Paragraph::new("Press 'q' to quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[5]);
}
