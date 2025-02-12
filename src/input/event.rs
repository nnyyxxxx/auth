use crate::{AuthResult, utils::constants::EVENT_POLL_DURATION};
use ratatui::crossterm::event::{self, Event};
use std::time::Duration;

pub fn poll_event() -> AuthResult<Option<Event>> {
    if event::poll(Duration::from_millis(EVENT_POLL_DURATION))? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
