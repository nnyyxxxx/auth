use crate::{
    auth_core::app::App,
    utils::{constants::FIRST_ENTRY_ROW, error::AuthResult},
};
use ratatui::crossterm::event::{MouseEvent, MouseEventKind};

pub fn handle_mouse_event(app: &mut App, event: MouseEvent) -> AuthResult<()> {
    match event.kind {
        MouseEventKind::Down(_) => handle_mouse_click(app),
        MouseEventKind::Moved => handle_mouse_hover(app, event.row as usize),
        _ => Ok(()),
    }
}

fn handle_mouse_click(app: &mut App) -> AuthResult<()> {
    app.copy_current_code()
}

fn handle_mouse_hover(app: &mut App, row: usize) -> AuthResult<()> {
    if row >= FIRST_ENTRY_ROW && row < app.entries.len() + FIRST_ENTRY_ROW {
        app.selected = row - FIRST_ENTRY_ROW;
    }
    Ok(())
}
