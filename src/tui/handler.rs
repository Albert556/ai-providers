use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::profile::manager::ProfileSource;

use super::app::{AddProfileState, AddStep, App, Mode, StatusKind};

pub enum Action {
    None,
    Quit,
    SuspendForEditor(String),
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    // Global: Ctrl+C quits from anywhere
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Action::Quit;
    }

    match &app.mode {
        Mode::ProfileList => handle_profile_list(app, key),
        Mode::ProfileDetail { .. } => handle_profile_detail(app, key),
        Mode::ActiveConfig { .. } => handle_active_config(app, key),
        Mode::ConfirmDelete { .. } => handle_confirm_delete(app, key),
        Mode::AddProfile(_) => handle_add_profile(app, key),
    }
}

fn handle_profile_list(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down();
            Action::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
            Action::None
        }
        KeyCode::Enter => {
            if let Some(name) = app.selected_profile().map(|s| s.to_string()) {
                app.mode = Mode::ProfileDetail {
                    name,
                    merged: false,
                    scroll: 0,
                };
            }
            Action::None
        }
        KeyCode::Char('u') => {
            app.use_selected();
            Action::None
        }
        KeyCode::Char('d') => {
            if let Some(name) = app.selected_profile().map(|s| s.to_string()) {
                app.mode = Mode::ConfirmDelete { profile: name };
            }
            Action::None
        }
        KeyCode::Char('e') => {
            if let Some(name) = app.selected_profile().map(|s| s.to_string()) {
                return Action::SuspendForEditor(name);
            }
            Action::None
        }
        KeyCode::Char('c') => {
            app.mode = Mode::ActiveConfig { scroll: 0 };
            Action::None
        }
        KeyCode::Char('a') => {
            app.mode = Mode::AddProfile(AddProfileState::new());
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_profile_detail(app: &mut App, key: KeyEvent) -> Action {
    // We need to read current detail state before mutating
    let (name, merged, scroll) = match &app.mode {
        Mode::ProfileDetail {
            name,
            merged,
            scroll,
        } => (name.clone(), *merged, *scroll),
        _ => return Action::None,
    };

    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::ProfileList;
            Action::None
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.mode = Mode::ProfileDetail {
                name,
                merged,
                scroll: scroll.saturating_add(1),
            };
            Action::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.mode = Mode::ProfileDetail {
                name,
                merged,
                scroll: scroll.saturating_sub(1),
            };
            Action::None
        }
        KeyCode::Char('m') => {
            app.mode = Mode::ProfileDetail {
                name,
                merged: !merged,
                scroll: 0,
            };
            Action::None
        }
        KeyCode::Char('u') => {
            match app.manager.use_profile(&name) {
                Ok(()) => {
                    app.set_status(StatusKind::Success, format!("Switched to '{}'", name));
                    let _ = app.refresh();
                }
                Err(e) => {
                    app.set_status(StatusKind::Error, format!("Error: {}", e));
                }
            }
            app.mode = Mode::ProfileList;
            Action::None
        }
        KeyCode::Char('e') => Action::SuspendForEditor(name),
        KeyCode::Char('d') => {
            app.mode = Mode::ConfirmDelete { profile: name };
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_active_config(app: &mut App, key: KeyEvent) -> Action {
    let scroll = match &app.mode {
        Mode::ActiveConfig { scroll } => *scroll,
        _ => return Action::None,
    };

    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::ProfileList;
            Action::None
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.mode = Mode::ActiveConfig {
                scroll: scroll.saturating_add(1),
            };
            Action::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.mode = Mode::ActiveConfig {
                scroll: scroll.saturating_sub(1),
            };
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_confirm_delete(app: &mut App, key: KeyEvent) -> Action {
    let profile = match &app.mode {
        Mode::ConfirmDelete { profile } => profile.clone(),
        _ => return Action::None,
    };

    match key.code {
        KeyCode::Char('y') => {
            app.delete_confirmed(&profile);
            Action::None
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.mode = Mode::ProfileList;
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_add_profile(app: &mut App, key: KeyEvent) -> Action {
    let state = match &app.mode {
        Mode::AddProfile(s) => s.clone(),
        _ => return Action::None,
    };

    match state.step {
        AddStep::EnterName => handle_add_enter_name(app, key, state),
        AddStep::SelectSource => handle_add_select_source(app, key, state),
    }
}

fn handle_add_enter_name(app: &mut App, key: KeyEvent, mut state: AddProfileState) -> Action {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::ProfileList;
        }
        KeyCode::Enter => {
            let name = state.name_input.trim().to_string();
            if name.is_empty() {
                app.set_status(StatusKind::Error, "Profile name cannot be empty");
            } else if let Err(e) = app.manager.validate_profile_name(&name) {
                app.set_status(StatusKind::Error, format!("{}", e));
            } else if app.manager.profile_exists(&name) {
                app.set_status(
                    StatusKind::Error,
                    format!("Profile '{}' already exists", name),
                );
            } else {
                state.step = AddStep::SelectSource;
                state.source_selected = 0;
                app.mode = Mode::AddProfile(state);
                return Action::None;
            }
        }
        KeyCode::Char(c) => {
            state.name_input.push(c);
            app.mode = Mode::AddProfile(state);
        }
        KeyCode::Backspace => {
            state.name_input.pop();
            app.mode = Mode::AddProfile(state);
        }
        _ => {}
    }
    Action::None
}

fn handle_add_select_source(app: &mut App, key: KeyEvent, mut state: AddProfileState) -> Action {
    let num_options = if app.profiles.is_empty() {
        // If no profiles, "From profile..." is not available
        2
    } else {
        AddProfileState::source_options().len()
    };

    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::ProfileList;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if state.source_selected < num_options - 1 {
                state.source_selected += 1;
            }
            app.mode = Mode::AddProfile(state);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if state.source_selected > 0 {
                state.source_selected -= 1;
            }
            app.mode = Mode::AddProfile(state);
        }
        KeyCode::Enter => {
            let name = state.name_input.clone();
            let source = match state.source_selected {
                0 => ProfileSource::FromCurrent,
                1 => ProfileSource::Empty,
                2 => {
                    // Use currently selected profile in list as source
                    if let Some(from) = app.selected_profile().map(|s| s.to_string()) {
                        ProfileSource::FromProfile(from)
                    } else {
                        app.set_status(StatusKind::Error, "No profile selected to copy from");
                        app.mode = Mode::ProfileList;
                        return Action::None;
                    }
                }
                _ => ProfileSource::Empty,
            };
            app.add_profile(&name, source);
        }
        _ => {}
    }
    Action::None
}
