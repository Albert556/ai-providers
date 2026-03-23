mod app;
mod event;
mod handler;
mod ui;

use std::io;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::profile::manager::ProfileManager;
use crate::util;

use app::{App, Mode, StatusKind};
use handler::Action;

pub fn run_tui(manager: &ProfileManager) -> Result<()> {
    install_panic_hook();

    let mut terminal = setup_terminal()?;
    let result = run_loop(&mut terminal, manager);
    restore_terminal(&mut terminal)?;
    result
}

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).context("Failed to create terminal")
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode().context("Failed to disable raw mode")?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;
    Ok(())
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    manager: &ProfileManager,
) -> Result<()> {
    let mut app = App::new(manager)?;

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if let Some(key) = event::poll_key_event(Duration::from_millis(250))? {
            match handler::handle_key(&mut app, key) {
                Action::None => {}
                Action::Quit => break,
                Action::SuspendForEditor(profile) => {
                    suspend_for_editor(terminal, &mut app, &profile)?;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn suspend_for_editor(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    profile: &str,
) -> Result<()> {
    if !app.manager.profile_exists(profile) {
        app.set_status(
            StatusKind::Error,
            format!("Profile '{}' not found", profile),
        );
        return Ok(());
    }

    // Leave TUI
    restore_terminal(terminal)?;

    let editor = util::resolve_editor();
    let profile_path = app.manager.profile_path(profile);

    let status = Command::new(&editor)
        .arg(&profile_path)
        .status()
        .with_context(|| format!("Failed to launch editor '{}'", editor))?;

    // Re-enter TUI
    enable_raw_mode().context("Failed to re-enable raw mode")?;
    crossterm::execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )
    .context("Failed to re-enter alternate screen")?;
    terminal.clear()?;

    if status.success() {
        // Validate JSON
        match app.manager.get_profile(profile) {
            Ok(_) => {
                app.set_status(StatusKind::Success, format!("Profile '{}' saved", profile));
            }
            Err(e) => {
                app.set_status(
                    StatusKind::Error,
                    format!("Warning: invalid JSON in '{}': {}", profile, e),
                );
            }
        }
    } else {
        app.set_status(StatusKind::Error, "Editor exited with non-zero status");
    }

    let _ = app.refresh();
    app.mode = Mode::ProfileList;
    Ok(())
}
