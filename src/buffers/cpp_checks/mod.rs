// Copyright (C) 2026 CharOfString <root@charofstring.cc>
//
//
// This software is free software: you can redistribute it and/or modify it under the terms of the
// GNU General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This software is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with this software. If
// not, see <https://www.gnu.org/licenses/>.

pub(super) mod settings;
#[cfg(test)]
mod tests;
mod ui;

use crate::app::App;
use settings::{Settings, settings_path};
use std::{io, path::PathBuf, process::Command, sync::mpsc};

#[derive(Clone, Copy)]
pub(crate) enum Tool {
    Tidy,
    Cpplint,
}

impl Tool {
    fn name(self) -> &'static str {
        match self {
            Self::Tidy => "Clang-tidy",
            Self::Cpplint => "Cpplint",
        }
    }
}

#[derive(Default)]
pub(crate) struct Checks {
    pub(crate) settings: Settings,
    config_path: Option<PathBuf>,
    pub(crate) path_input: Option<String>,
    pending: Option<mpsc::Receiver<String>>,
    report: Option<String>,
    scroll: u16,
}

impl App {
    pub(crate) fn load_cpp_preferences(&mut self) {
        self.cpp_checks.config_path = settings_path();
        if let Some(path) = &self.cpp_checks.config_path {
            match Settings::load(path) {
                Ok(settings) => self.cpp_checks.settings = settings,
                Err(error) => self.status_bar_text = format!("C++ preferences: {error}"),
            }
        }
    }

    pub(crate) fn is_cpp(&self) -> bool {
        self.buffer.editor.code_ref().lang() == "cpp"
    }

    fn save_cpp_preferences(&mut self, settings: Settings) {
        let result = self.cpp_checks.config_path.as_ref().map_or_else(
            || Err(io::Error::other("No configuration directory available")),
            |path| settings.save(path),
        );
        match result {
            Ok(()) => {
                self.cpp_checks.settings = settings;
                self.cpp_checks.path_input = None;
                self.status_bar_text = "C++ preferences saved".into();
            }
            Err(error) => self.status_bar_text = format!("Cannot save C++ preferences: {error}"),
        }
    }

    pub(crate) fn toggle_cpp_check(&mut self, tool: Tool) {
        if !self.is_cpp() {
            return;
        }

        let mut settings = self.cpp_checks.settings.clone();
        match tool {
            Tool::Tidy => settings.tidy = !settings.tidy,
            Tool::Cpplint => {
                if !settings.cpplint && settings.cpplint_path.is_empty() {
                    self.completion.close();
                    self.cpp_checks.path_input = Some(String::new());
                    self.status_bar_text = "Enter the cpplint executable path".into();
                    return;
                }
                settings.cpplint = !settings.cpplint;
            }
        }
        self.save_cpp_preferences(settings);
    }

    pub(crate) fn run_cpp_check(&mut self, tool: Tool) {
        if !self.is_cpp() {
            self.status_bar_text = "This check is only available in C++ mode".into();
            return;
        }
        let settings = &self.cpp_checks.settings;
        let enabled = match tool {
            Tool::Tidy => settings.tidy,
            Tool::Cpplint => settings.cpplint,
        };
        let name = tool.name();
        if !enabled {
            self.status_bar_text = format!("{name} is disabled; enable it in the LSP menu");
            return;
        }
        if self.cpp_checks.pending.is_some() {
            self.status_bar_text = "A C++ check is already running".into();
            return;
        }
        self.buffer.check_disk();
        if self.buffer.path.is_none() || self.buffer.is_dirty() || self.buffer.obsolete {
            self.status_bar_text = "Save the current buffer before checking".into();
            return;
        }
        let path = match self.buffer.path.as_ref().unwrap().canonicalize() {
            Ok(path) => path,
            Err(error) => {
                self.status_bar_text = format!("Cannot check file: {error}");
                return;
            }
        };

        let program = match tool {
            Tool::Tidy => "clang-tidy".to_owned(),
            Tool::Cpplint => settings.cpplint_path.clone(),
        };
        let (sender, receiver) = mpsc::channel();

        // Run without a shell so spaces and shell characters in paths stay literal.
        std::thread::spawn(move || {
            let result = Command::new(program).arg(&path).output();
            let report = match result {
                Ok(output) => format!(
                    "{name}: {}\n{}\n\n{}{}",
                    output.status,
                    path.display(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                ),
                Err(error) => format!("{name} failed: {error}\n{}", path.display()),
            };
            let _ = sender.send(report);
        });
        self.cpp_checks.pending = Some(receiver);
        self.status_bar_text = format!("Running {name}…");
    }

    pub(crate) fn poll_cpp_check(&mut self) {
        // Do not interrupt a save or setup prompt with the result window.
        if self.save_path_input.is_some()
            || self.overwrite_confirm
            || self.cpp_checks.path_input.is_some()
            || self.menu.active.is_some()
        {
            return;
        }
        let Some(receiver) = &self.cpp_checks.pending else {
            return;
        };
        let report = match receiver.try_recv() {
            Ok(report) => report,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => "C++ check worker stopped".into(),
        };
        self.cpp_checks.pending = None;
        self.completion.close();
        self.cpp_checks.report = Some(report);
        self.cpp_checks.scroll = 0;
        self.status_bar_text = "C++ check finished".into();
    }
}
