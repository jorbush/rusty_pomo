use clap::ValueEnum;
use ratatui::style::Color;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Theme {
    Dracula,
    SolarizedDark,
    GruvboxDark,
}

impl Theme {
    pub fn colors(self) -> (Color, Color, Color) {
        match self {
            Theme::Dracula => (Color::Rgb(40, 42, 54), Color::Rgb(189, 147, 249), Color::Rgb(80, 250, 123)),
            Theme::SolarizedDark => (Color::Rgb(0, 43, 54), Color::Rgb(38, 139, 210), Color::Rgb(133, 153, 0)),
            Theme::GruvboxDark => (Color::Rgb(40, 40, 40), Color::Rgb(250, 189, 47), Color::Rgb(184, 187, 38)),
        }
    }
}
