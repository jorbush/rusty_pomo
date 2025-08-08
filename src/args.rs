use clap::{Parser};

use crate::theme::Theme;

#[derive(Parser, Debug)]
#[command(name = "rusty_pomo", about = "Minimalist, visually pleasing Pomodoro CLI", version)]
pub struct Args {
    /// Focus minutes
    #[arg(short = 'f', long, default_value_t = 25)]
    pub focus: u64,
    /// Short break minutes
    #[arg(short = 's', long, default_value_t = 5)]
    pub short: u64,
    /// Long break minutes
    #[arg(short = 'l', long, default_value_t = 15)]
    pub long: u64,
    /// Number of focus sessions before long break
    #[arg(short = 'n', long, default_value_t = 4)]
    pub long_every: u64,
    /// Theme
    #[arg(long, value_enum, default_value_t = Theme::Dracula)]
    pub theme: Theme,
    /// Enable desktop notifications
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub notifications: bool,
    /// Notification sound name (platform-dependent). Example macOS: Ping, Submarine. Linux: message-new-instant
    #[arg(long)]
    pub notification_sound: Option<String>,
    /// Notification duration in seconds (if supported by OS; macOS ignores)
    #[arg(long, default_value_t = 10)]
    pub notification_seconds: u64,
    /// macOS only: bundle identifier to use for notifications (controls icon). Requires the app to be installed with this bundle id and icon.
    #[arg(long)]
    pub macos_bundle_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults() {
        let args = Args::parse_from(["rusty_pomo"]);
        assert_eq!(args.focus, 25);
        assert_eq!(args.short, 5);
        assert_eq!(args.long, 15);
        assert_eq!(args.long_every, 4);
        assert!(args.notifications);
        assert_eq!(args.notification_seconds, 10);
    }

    #[test]
    fn parses_overrides() {
        let args = Args::parse_from([
            "rusty_pomo",
            "--focus",
            "50",
            "--short",
            "10",
            "--long",
            "20",
            "--long-every",
            "3",
            "--notifications=false",
            "--notification-seconds",
            "15",
        ]);
        assert_eq!(args.focus, 50);
        assert_eq!(args.short, 10);
        assert_eq!(args.long, 20);
        assert_eq!(args.long_every, 3);
        assert!(!args.notifications);
        assert_eq!(args.notification_seconds, 15);
    }
}
