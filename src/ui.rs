use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};

use crate::state::{AppState, PhaseKind};

pub fn draw(frame: &mut Frame, app: &AppState) {
    let (bg, accent, ok) = app.theme.colors();
    let size = frame.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(3),
        ])
        .split(size);

    // Header
    let title = match app.current_phase.kind {
        PhaseKind::Focus => ("Focus", accent),
        PhaseKind::ShortBreak => ("Short Break", ok),
        PhaseKind::LongBreak => ("Long Break", ok),
    };
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "Rusty Pomo · ",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            title.0,
            Style::default().fg(title.1).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(header, chunks[0]);

    // Timer + Gauge
    let remaining = app.time_remaining(std::time::Instant::now());
    let progress = app.progress(std::time::Instant::now());
    let timer_text = format_mm_ss(remaining);
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(title.1))
        .ratio(progress)
        .label(Span::styled(
            timer_text,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
    let gauge_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled("Session", Style::default().fg(Color::Gray)));
    frame.render_widget(gauge_block, chunks[1]);
    frame.render_widget(gauge, chunks[1]);

    // Footer / Help
    let help = Paragraph::new(vec![Line::from(vec![
        Span::styled("␣ ", Style::default().fg(Color::Gray)),
        Span::styled("pause/resume  ", Style::default().fg(Color::White)),
        Span::styled("n ", Style::default().fg(Color::Gray)),
        Span::styled("next  ", Style::default().fg(Color::White)),
        Span::styled("r ", Style::default().fg(Color::Gray)),
        Span::styled("reset  ", Style::default().fg(Color::White)),
        Span::styled("q ", Style::default().fg(Color::Gray)),
        Span::styled("quit", Style::default().fg(Color::White)),
    ])])
    .wrap(Wrap { trim: true })
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray))
            .style(Style::default().bg(bg)),
    );
    frame.render_widget(help, chunks[2]);
}

pub fn format_mm_ss(d: std::time::Duration) -> String {
    let total = d.as_secs();
    let minutes = total / 60;
    let seconds = total % 60;
    format!("{minutes:02}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn formats_mm_ss() {
        assert_eq!(format_mm_ss(Duration::from_secs(0)), "00:00");
        assert_eq!(format_mm_ss(Duration::from_secs(59)), "00:59");
        assert_eq!(format_mm_ss(Duration::from_secs(60)), "01:00");
        assert_eq!(format_mm_ss(Duration::from_secs(125)), "02:05");
    }
}
