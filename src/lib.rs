use anyhow::Result;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;

pub mod app;
pub mod entry;
pub mod ui;
pub mod utils;

pub use app::App;
pub use utils::{shutdown, startup};

pub fn run() -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = App::new()?;

    while !app.should_quit {
        terminal.draw(|f| ui::draw(f, &app))?;
        app.handle_events()?;
    }

    Ok(())
}