mod args;
mod notifications;
mod run;
mod state;
mod theme;
mod ui;

use clap::Parser;
use std::io;

use crate::args::Args;
use crate::notifications::maybe_init_macos_bundle;
use crate::run::run;
use crate::state::AppState;

fn main() -> io::Result<()> {
    let args = Args::parse();
    maybe_init_macos_bundle(&args);
    let app = AppState::new(args);
    run(app)
}
