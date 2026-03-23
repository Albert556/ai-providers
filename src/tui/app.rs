use anyhow::Result;

use crate::profile::manager::{ProfileManager, ProfileSource};

#[derive(Clone)]
pub enum StatusKind {
    Info,
    Success,
    Error,
}

pub enum Mode {
    ProfileList,
    ProfileDetail {
        name: String,
        merged: bool,
        scroll: u16,
    },
    ActiveConfig {
        scroll: u16,
    },
    AddProfile(AddProfileState),
    ConfirmDelete {
        profile: String,
    },
}

#[derive(Clone)]
pub enum AddStep {
    EnterName,
    SelectSource,
}

#[derive(Clone)]
pub struct AddProfileState {
    pub name_input: String,
    pub step: AddStep,
    pub source_selected: usize,
}

impl AddProfileState {
    pub fn new() -> Self {
        Self {
            name_input: String::new(),
            step: AddStep::EnterName,
            source_selected: 0,
        }
    }

    pub fn source_options() -> &'static [&'static str] {
        &["Current config", "Empty", "From profile..."]
    }
}

pub struct App<'a> {
    pub manager: &'a ProfileManager<'a>,
    pub mode: Mode,
    pub profiles: Vec<String>,
    pub current_profile: Option<String>,
    pub has_common: bool,
    pub selected: usize,
    pub status_message: Option<(StatusKind, String)>,
    pub should_quit: bool,
}

impl<'a> App<'a> {
    pub fn new(manager: &'a ProfileManager<'a>) -> Result<Self> {
        let mut profiles = manager.list_profiles()?;
        let current_profile = manager.get_current_profile()?;
        let has_common = manager.has_common_config();
        if has_common {
            profiles.push("common".to_string());
        }

        Ok(Self {
            manager,
            mode: Mode::ProfileList,
            profiles,
            current_profile,
            has_common,
            selected: 0,
            status_message: Some((
                StatusKind::Info,
                "Ready. Press 'a' to add a profile or Enter to inspect one.".to_string(),
            )),
            should_quit: false,
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.profiles = self.manager.list_profiles()?;
        self.current_profile = self.manager.get_current_profile()?;
        self.has_common = self.manager.has_common_config();
        if self.has_common {
            self.profiles.push("common".to_string());
        }
        if self.selected >= self.profiles.len() && !self.profiles.is_empty() {
            self.selected = self.profiles.len() - 1;
        }
        Ok(())
    }

    pub fn selected_profile(&self) -> Option<&str> {
        self.profiles.get(self.selected).map(|s| s.as_str())
    }

    pub fn set_status(&mut self, kind: StatusKind, msg: impl Into<String>) {
        self.status_message = Some((kind, msg.into()));
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.profiles.is_empty() && self.selected < self.profiles.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn use_selected(&mut self) {
        let Some(name) = self.selected_profile().map(|s| s.to_string()) else {
            return;
        };
        if ProfileManager::is_common_profile(&name) {
            self.set_status(
                StatusKind::Error,
                "'common' is a shared base config, not a switchable profile",
            );
            return;
        }
        match self.manager.use_profile(&name) {
            Ok(()) => {
                self.set_status(StatusKind::Success, format!("Switched to '{}'", name));
                let _ = self.refresh();
            }
            Err(e) => {
                self.set_status(StatusKind::Error, format!("Error: {}", e));
            }
        }
    }

    pub fn delete_confirmed(&mut self, profile: &str) {
        let profile = profile.to_string();
        match self.manager.delete_profile(&profile) {
            Ok(()) => {
                self.set_status(StatusKind::Success, format!("Deleted '{}'", profile));
                let _ = self.refresh();
            }
            Err(e) => {
                self.set_status(StatusKind::Error, format!("Error: {}", e));
            }
        }
        self.mode = Mode::ProfileList;
    }

    pub fn add_profile(&mut self, name: &str, source: ProfileSource) {
        match self.manager.add_profile(name, source) {
            Ok(()) => {
                self.set_status(StatusKind::Success, format!("Added '{}'", name));
                let _ = self.refresh();
                // Select the newly added profile
                if let Some(idx) = self.profiles.iter().position(|p| p == name) {
                    self.selected = idx;
                }
            }
            Err(e) => {
                self.set_status(StatusKind::Error, format!("Error: {}", e));
            }
        }
        self.mode = Mode::ProfileList;
    }
}
