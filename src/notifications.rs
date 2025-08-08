use std::path::Path;
use std::time::Duration;
use notify_rust::Notification;
#[cfg(target_os = "macos")]
use notify_rust::set_application;
use crate::state::{AppState, PhaseKind};

#[allow(unused_variables)] // Because it's only used on macOS
pub fn maybe_init_macos_bundle(args: &crate::args::Args) {
    #[cfg(target_os = "macos")]
    if let Some(bundle_id) = &args.macos_bundle_id {
        let _ = set_application(bundle_id);
    }
}

pub fn maybe_notify(app: &AppState) {
    if !app.args.notifications {
        return;
    }

    let (title, body) = match app.current_phase.kind {
        PhaseKind::Focus => ("Focus", "Let’s get to work."),
        PhaseKind::ShortBreak => ("Short Break", "Time for a quick breather."),
        PhaseKind::LongBreak => ("Long Break", "Enjoy a longer rest."),
    };

    let mut n = Notification::new();
    n.summary(&format!("Rusty Pomo · {title}")).body(body);

    if let Some(icon_path) = asset_icon_path() {
        n.icon(&icon_path);
        #[cfg(target_os = "windows")]
        {
            n.image_path(&icon_path);
        }
    }

    if let Some(sound) = &app.args.notification_sound {
        n.sound_name(sound);
    } else {
        n.sound_name("default");
    }

    n.timeout(Duration::from_secs(app.args.notification_seconds));
    let _ = n.show();
}

fn asset_icon_path() -> Option<String> {
    let candidates = [
        format!("{}/docs/assets/rusty_pomo.png", env!("CARGO_MANIFEST_DIR")),
        "docs/assets/rusty_pomo.png".to_string(),
    ];
    for candidate in candidates.iter() {
        if Path::new(candidate).exists() {
            return Some(candidate.clone());
        }
    }
    None
}
