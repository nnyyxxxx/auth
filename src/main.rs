use auth::{
    auth_core::app::App,
    input::{event, linux, root},
    ui::core::draw,
    utils::{cli, error::AuthResult},
};
use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
};
use std::io::stdout;

fn main() -> AuthResult<()> {
    let args = cli::parse_args();

    if !args.no_linux_check && !linux::check_linux() {
        linux::show_linux_warning()?;
        return Ok(());
    }

    if !args.no_root_check && root::check_root() {
        root::show_root_warning()?;
        return Ok(());
    }

    if args.mouse {
        execute!(stdout(), EnableMouseCapture)?;
    }

    let mut terminal = ratatui::init();
    let mut app = App::new()?;

    while !app.should_quit {
        terminal.draw(|f| draw(f, &app, args.no_size_check))?;

        if let Some(event) = event::poll_event()? {
            app.handle_events(event)?;
        }
    }

    if args.mouse {
        execute!(stdout(), DisableMouseCapture)?;
    }

    ratatui::restore();
    Ok(())
}
