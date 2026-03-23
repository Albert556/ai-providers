use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::profile::manager::ProfileManager;

use super::app::{AddProfileState, AddStep, App, Mode, StatusKind};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Min(1),    // main content
        Constraint::Length(2), // status + help bar
    ])
    .split(f.area());

    match &app.mode {
        Mode::ProfileList => render_profile_list(f, app, chunks[0]),
        Mode::ProfileDetail {
            name,
            merged,
            scroll,
        } => render_profile_detail(f, app, chunks[0], name, *merged, *scroll),
        Mode::ActiveConfig { scroll } => render_active_config(f, app, chunks[0], *scroll),
        Mode::AddProfile(state) => {
            render_profile_list(f, app, chunks[0]);
            render_add_dialog(f, app, state);
        }
        Mode::ConfirmDelete { profile } => {
            render_profile_list(f, app, chunks[0]);
            render_confirm_delete(f, profile);
        }
    }

    render_status_bar(f, app, chunks[1]);
}

fn render_profile_list(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(
        " aip · {} ",
        capitalize_provider(app.manager.provider_name())
    );

    if app.profiles.is_empty() {
        let block = Block::default().title(title).borders(Borders::ALL);
        let text = Paragraph::new("No profiles found. Press 'a' to add one.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(text, area);
        return;
    }

    let items: Vec<ListItem> = app
        .profiles
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let is_current = app.current_profile.as_deref() == Some(name.as_str());
            let is_common = ProfileManager::is_common_profile(name);

            let cursor = if i == app.selected { "▸ " } else { "  " };

            let mut spans = vec![Span::raw(cursor)];

            let name_style = if i == app.selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            spans.push(Span::styled(name.clone(), name_style));

            if is_current {
                spans.push(Span::styled(
                    " [current]",
                    Style::default().fg(Color::Green),
                ));
            }
            if is_common {
                spans.push(Span::styled(" [base]", Style::default().fg(Color::Yellow)));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let block = Block::default().title(title).borders(Borders::ALL);
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_profile_detail(
    f: &mut Frame,
    app: &App,
    area: Rect,
    name: &str,
    merged: bool,
    scroll: u16,
) {
    let title = if merged {
        format!(" {} (merged with common) ", name)
    } else {
        format!(" {} ", name)
    };

    let content = if merged {
        match app.manager.get_profile(name) {
            Ok(profile) => match app.manager.get_common_config() {
                Ok(Some(common)) => {
                    let merged_val = crate::profile::storage::deep_merge(&common, &profile);
                    serde_json::to_string_pretty(&merged_val)
                        .unwrap_or_else(|e| format!("Error: {}", e))
                }
                Ok(None) => serde_json::to_string_pretty(&profile)
                    .unwrap_or_else(|e| format!("Error: {}", e)),
                Err(e) => format!("Error reading common config: {}", e),
            },
            Err(e) => format!("Error: {}", e),
        }
    } else {
        match app.manager.get_profile(name) {
            Ok(val) => {
                serde_json::to_string_pretty(&val).unwrap_or_else(|e| format!("Error: {}", e))
            }
            Err(e) => format!("Error: {}", e),
        }
    };

    let block = Block::default().title(title).borders(Borders::ALL);
    let paragraph = Paragraph::new(content)
        .block(block)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn render_active_config(f: &mut Frame, app: &App, area: Rect, scroll: u16) {
    let title = " Active Config (settings.json) ";
    let content = match app.manager.get_active_config() {
        Ok(val) => serde_json::to_string_pretty(&val).unwrap_or_else(|e| format!("Error: {}", e)),
        Err(e) => format!("Error: {}", e),
    };

    let block = Block::default().title(title).borders(Borders::ALL);
    let paragraph = Paragraph::new(content)
        .block(block)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(1), // status message
        Constraint::Length(1), // help bar
    ])
    .split(area);

    // Status message
    if let Some((kind, msg)) = &app.status_message {
        let color = match kind {
            StatusKind::Info => Color::Blue,
            StatusKind::Success => Color::Green,
            StatusKind::Error => Color::Red,
        };
        let status = Paragraph::new(Span::styled(
            format!(" {}", msg),
            Style::default().fg(color),
        ));
        f.render_widget(status, chunks[0]);
    }

    // Help bar
    let help_text = match &app.mode {
        Mode::ProfileList => {
            " q:Quit  j/k:Navigate  Enter:View  u:Use  a:Add  d:Delete  e:Edit  c:Config"
        }
        Mode::ProfileDetail { .. } => {
            " Esc:Back  j/k:Scroll  m:Toggle merged  u:Use  e:Edit  d:Delete"
        }
        Mode::ActiveConfig { .. } => " Esc:Back  j/k:Scroll",
        Mode::ConfirmDelete { .. } => " y:Confirm  n/Esc:Cancel",
        Mode::AddProfile(state) => match state.step {
            AddStep::EnterName => " Esc:Cancel  Enter:Confirm name",
            AddStep::SelectSource => " Esc:Cancel  j/k:Navigate  Enter:Select",
        },
    };

    let help = Paragraph::new(Span::styled(
        help_text,
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    ));
    f.render_widget(help, chunks[1]);
}

fn render_confirm_delete(f: &mut Frame, profile: &str) {
    let area = centered_rect(50, 7, f.area());
    f.render_widget(Clear, area);

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Delete profile "),
            Span::styled(
                format!("'{}'", profile),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("?"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  y: Yes  n: No",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let block = Block::default()
        .title(" Confirm Delete ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red));
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn render_add_dialog(f: &mut Frame, app: &App, state: &AddProfileState) {
    let area = centered_rect(50, 10, f.area());
    f.render_widget(Clear, area);

    match state.step {
        AddStep::EnterName => {
            let text = vec![
                Line::from(""),
                Line::from(format!("  Name: {}_", state.name_input)),
                Line::from(""),
                Line::from(Span::styled(
                    "  Enter: Confirm  Esc: Cancel",
                    Style::default().fg(Color::DarkGray),
                )),
            ];

            let block = Block::default()
                .title(" Add Profile ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Cyan));
            let paragraph = Paragraph::new(text).block(block);
            f.render_widget(paragraph, area);
        }
        AddStep::SelectSource => {
            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("  Source for '{}':", state.name_input),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            let options = AddProfileState::source_options();
            let num_options = if app.profiles.is_empty() {
                2
            } else {
                options.len()
            };

            for (i, opt) in options.iter().take(num_options).enumerate() {
                let cursor = if i == state.source_selected {
                    "  ▸ "
                } else {
                    "    "
                };
                let style = if i == state.source_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let mut label = format!("{}{}", cursor, opt);
                if i == 2 {
                    // "From profile..." - show which profile
                    if let Some(selected) = app.selected_profile() {
                        label = format!("{}From '{}'", cursor, selected);
                    }
                }
                lines.push(Line::from(Span::styled(label, style)));
            }

            let block = Block::default()
                .title(" Add Profile - Select Source ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Cyan));
            let paragraph = Paragraph::new(lines).block(block);
            f.render_widget(paragraph, area);
        }
    }
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let popup_width = area.width * percent_x / 100;
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    Rect::new(
        area.x + x,
        area.y + y,
        popup_width.min(area.width),
        height.min(area.height),
    )
}

fn capitalize_provider(name: &str) -> String {
    match name {
        "claude" => "Claude Code".to_string(),
        other => {
            let mut c = other.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }
    }
}
