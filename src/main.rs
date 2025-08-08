use std::io::{self};
use std::path::Path;
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};
use ratatui::Terminal;
use notify_rust::Notification;
#[cfg(target_os = "macos")]
use notify_rust::set_application;

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Theme {
    Dracula,
    SolarizedDark,
    GruvboxDark,
}

impl Theme {
    fn colors(self) -> (Color, Color, Color) {
        match self {
            Theme::Dracula => (Color::Rgb(40, 42, 54), Color::Rgb(189, 147, 249), Color::Rgb(80, 250, 123)),
            Theme::SolarizedDark => (Color::Rgb(0, 43, 54), Color::Rgb(38, 139, 210), Color::Rgb(133, 153, 0)),
            Theme::GruvboxDark => (Color::Rgb(40, 40, 40), Color::Rgb(250, 189, 47), Color::Rgb(184, 187, 38)),
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "rusty_pomo", about = "Minimalist, visually pleasing Pomodoro CLI", version)]
struct Args {
    /// Focus minutes
    #[arg(short = 'f', long, default_value_t = 25)]
    focus: u64,
    /// Short break minutes
    #[arg(short = 's', long, default_value_t = 5)]
    short: u64,
    /// Long break minutes
    #[arg(short = 'l', long, default_value_t = 15)]
    long: u64,
    /// Number of focus sessions before long break
    #[arg(short = 'n', long, default_value_t = 4)]
    long_every: u64,
    /// Theme
    #[arg(long, value_enum, default_value_t = Theme::Dracula)]
    theme: Theme,
    /// Enable desktop notifications
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    notifications: bool,
    /// Notification sound name (platform-dependent). Example macOS: Ping, Submarine. Linux: message-new-instant
    #[arg(long)]
    notification_sound: Option<String>,
    /// Notification duration in seconds (if supported by OS; macOS ignores)
    #[arg(long, default_value_t = 10)]
    notification_seconds: u64,
    /// macOS only: bundle identifier to use for notifications (controls icon). Requires the app to be installed with this bundle id and icon.
    #[arg(long)]
    macos_bundle_id: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PhaseKind {
    Focus,
    ShortBreak,
    LongBreak,
}

#[derive(Clone, Debug)]
struct Phase {
    kind: PhaseKind,
    duration: Duration,
}

#[derive(Debug)]
struct AppState {
    args: Args,
    theme: Theme,
    session_index: u64,
    current_phase: Phase,
    phase_started_at: Instant,
    paused: bool,
    paused_at: Option<Instant>,
}

impl AppState {
    fn new(args: Args) -> Self {
        let theme = args.theme;
        let current_phase = Phase { kind: PhaseKind::Focus, duration: Duration::from_secs(args.focus * 60) };
        Self {
            args,
            theme,
            session_index: 0,
            current_phase,
            phase_started_at: Instant::now(),
            paused: false,
            paused_at: None,
        }
    }

    fn elapsed_in_phase(&self, now: Instant) -> Duration {
        if self.paused {
            if let Some(paused_at) = self.paused_at { return paused_at.saturating_duration_since(self.phase_started_at); }
            return Duration::from_secs(0);
        }
        now.saturating_duration_since(self.phase_started_at)
    }

    fn time_remaining(&self, now: Instant) -> Duration {
        self.current_phase.duration.saturating_sub(self.elapsed_in_phase(now))
    }

    fn progress(&self, now: Instant) -> f64 {
        let elapsed = self.elapsed_in_phase(now).as_secs_f64();
        let total = self.current_phase.duration.as_secs_f64().max(1.0);
        (elapsed / total).clamp(0.0, 1.0)
    }

    fn toggle_pause(&mut self) {
        if self.paused {
            if let Some(paused_at) = self.paused_at.take() {
                let paused_duration = Instant::now().saturating_duration_since(paused_at);
                self.phase_started_at += paused_duration;
            }
            self.paused = false;
        } else {
            self.paused = true;
            self.paused_at = Some(Instant::now());
        }
    }

    fn skip(&mut self) {
        self.advance_phase();
    }

    fn reset_phase(&mut self) {
        self.phase_started_at = Instant::now();
        self.paused = false;
        self.paused_at = None;
    }

    fn advance_phase(&mut self) {
        let next_kind = match self.current_phase.kind {
            PhaseKind::Focus => {
                self.session_index += 1;
                if self.session_index % self.args.long_every == 0 { PhaseKind::LongBreak } else { PhaseKind::ShortBreak }
            }
            PhaseKind::ShortBreak | PhaseKind::LongBreak => PhaseKind::Focus,
        };
        self.current_phase = match next_kind {
            PhaseKind::Focus => Phase { kind: PhaseKind::Focus, duration: Duration::from_secs(self.args.focus * 60) },
            PhaseKind::ShortBreak => Phase { kind: PhaseKind::ShortBreak, duration: Duration::from_secs(self.args.short * 60) },
            PhaseKind::LongBreak => Phase { kind: PhaseKind::LongBreak, duration: Duration::from_secs(self.args.long * 60) },
        };
        self.reset_phase();
        self.maybe_notify();
    }
}

impl AppState {
    fn maybe_notify(&self) {
        if !self.args.notifications { return; }
        let (title, body) = match self.current_phase.kind {
            PhaseKind::Focus => ("Focus", "Let’s get to work."),
            PhaseKind::ShortBreak => ("Short Break", "Time for a quick breather."),
            PhaseKind::LongBreak => ("Long Break", "Enjoy a longer rest."),
        };
        let mut n = Notification::new();
        n.summary(&format!("Rusty Pomo · {title}")).body(body);
        if let Some(icon_path) = asset_icon_path() {
            // Best effort: Windows/Linux honor icon; macOS ignores
            n.icon(&icon_path);
            #[cfg(target_os = "windows")]
            {
                n.image_path(&icon_path);
            }
        }
        if let Some(sound) = &self.args.notification_sound {
            n.sound_name(sound);
        } else {
            // A reasonable default across platforms
            n.sound_name("default");
        }
        n.timeout(Duration::from_secs(self.args.notification_seconds));
        let _ = n.show();
    }
}

fn asset_icon_path() -> Option<String> {
    let candidates = [
        format!("{}/assets/rusty_pomo.png", env!("CARGO_MANIFEST_DIR")),
        "assets/rusty_pomo.png".to_string(),
    ];
    for candidate in candidates.iter() {
        if Path::new(candidate).exists() {
            return Some(candidate.clone());
        }
    }
    None
}

fn format_mm_ss(d: Duration) -> String {
    let total = d.as_secs();
    let minutes = total / 60;
    let seconds = total % 60;
    format!("{minutes:02}:{seconds:02}")
}

fn run(mut app: AppState) -> io::Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
        let now = Instant::now();

        terminal.draw(|frame| {
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
                Span::styled("Rusty Pomo · ", Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
                Span::styled(title.0, Style::default().fg(title.1).add_modifier(Modifier::BOLD)),
            ]))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
            frame.render_widget(header, chunks[0]);

            // Timer + Gauge
            let remaining = app.time_remaining(now);
            let progress = app.progress(now) as f64;
            let timer_text = format_mm_ss(remaining);
            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(title.1))
                .ratio(progress)
                .label(Span::styled(timer_text, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
            let gauge_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Span::styled("Session", Style::default().fg(Color::Gray)));
            frame.render_widget(gauge_block, chunks[1]);
            frame.render_widget(gauge, chunks[1]);

            // Footer / Help
            let help = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("␣ ", Style::default().fg(Color::Gray)),
                    Span::styled("pause/resume  ", Style::default().fg(Color::White)),
                    Span::styled("n ", Style::default().fg(Color::Gray)),
                    Span::styled("next  ", Style::default().fg(Color::White)),
                    Span::styled("r ", Style::default().fg(Color::Gray)),
                    Span::styled("reset  ", Style::default().fg(Color::White)),
                    Span::styled("q ", Style::default().fg(Color::Gray)),
                    Span::styled("quit", Style::default().fg(Color::White)),
                ]),
            ])
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .style(Style::default().bg(bg)),
            );
            frame.render_widget(help, chunks[2]);
        })?;

        // Phase transitions
        if app.time_remaining(now).is_zero() && !app.paused {
            app.advance_phase();
        }

        // Input handling with tick
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(' ') => app.toggle_pause(),
                        KeyCode::Char('n') => app.skip(),
                        KeyCode::Char('r') => app.reset_phase(),
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // teardown
    terminal.show_cursor()?;
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    #[cfg(target_os = "macos")]
    if let Some(bundle_id) = &args.macos_bundle_id {
        // This makes macOS attribute notifications to the given app (and its icon)
        let _ = set_application(bundle_id);
    }
    let app = AppState::new(args);
    run(app)
}
