use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

/// Poll for a key event with a timeout.
pub fn poll_key_event(timeout: Duration) -> Result<Option<KeyEvent>> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            return Ok(Some(key));
        }
    }
    Ok(None)
}
