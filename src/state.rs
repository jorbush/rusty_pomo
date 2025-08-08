use std::time::{Duration, Instant};

use crate::args::Args;
use crate::theme::Theme;
use crate::notifications::maybe_notify;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PhaseKind {
    Focus,
    ShortBreak,
    LongBreak,
}

#[derive(Clone, Debug)]
pub struct Phase {
    pub kind: PhaseKind,
    pub duration: Duration,
}

#[derive(Debug)]
pub struct AppState {
    pub args: Args,
    pub theme: Theme,
    pub session_index: u64,
    pub current_phase: Phase,
    pub phase_started_at: Instant,
    pub paused: bool,
    pub paused_at: Option<Instant>,
}

impl AppState {
    pub fn new(args: Args) -> Self {
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

    pub fn elapsed_in_phase(&self, now: Instant) -> Duration {
        if self.paused {
            if let Some(paused_at) = self.paused_at { return paused_at.saturating_duration_since(self.phase_started_at); }
            return Duration::from_secs(0);
        }
        now.saturating_duration_since(self.phase_started_at)
    }

    pub fn time_remaining(&self, now: Instant) -> Duration {
        self.current_phase.duration.saturating_sub(self.elapsed_in_phase(now))
    }

    pub fn progress(&self, now: Instant) -> f64 {
        let elapsed = self.elapsed_in_phase(now).as_secs_f64();
        let total = self.current_phase.duration.as_secs_f64().max(1.0);
        (elapsed / total).clamp(0.0, 1.0)
    }

    pub fn toggle_pause(&mut self) {
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

    pub fn skip(&mut self) {
        self.advance_phase();
    }

    pub fn reset_phase(&mut self) {
        self.phase_started_at = Instant::now();
        self.paused = false;
        self.paused_at = None;
    }

    pub fn advance_phase(&mut self) {
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
        maybe_notify(self);
    }
}
