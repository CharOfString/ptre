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

use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Clear, Paragraph},
};

impl App {
    pub(crate) fn cpp_check_modal(&self) -> bool {
        self.cpp_checks.path_input.is_some() || self.cpp_checks.report.is_some()
    }

    pub(crate) fn handle_cpp_check_key(&mut self, key: KeyEvent) -> bool {
        if let Some(input) = &mut self.cpp_checks.path_input {
            match key.code {
                KeyCode::Esc => {
                    self.cpp_checks.path_input = None;
                    self.status_bar_text = "Cpplint setup cancelled".into();
                }
                KeyCode::Enter => {
                    let path = if let Some(rest) = input.strip_prefix("~/") {
                        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(rest))
                    } else {
                        Some(PathBuf::from(input.as_str()))
                    };
                    let path = path.and_then(|path| path.canonicalize().ok());
                    if let Some(path) = path.filter(|path| settings::executable(path)) {
                        let mut settings = self.cpp_checks.settings.clone();
                        settings.cpplint_path = path.to_string_lossy().into_owned();
                        settings.cpplint = true;
                        self.save_cpp_preferences(settings);
                    } else {
                        self.status_bar_text = "Enter a valid executable file path".into();
                    }
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(c)
                    if !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    input.push(c)
                }
                _ => {}
            }
            return true;
        }
        if let Some(report) = &self.cpp_checks.report {
            let max = report
                .lines()
                .count()
                .saturating_sub(1)
                .min(u16::MAX as usize) as u16;
            match key.code {
                KeyCode::Esc => self.cpp_checks.report = None,
                KeyCode::Up => self.cpp_checks.scroll = self.cpp_checks.scroll.saturating_sub(1),
                KeyCode::Down => {
                    self.cpp_checks.scroll = self.cpp_checks.scroll.saturating_add(1).min(max)
                }
                KeyCode::PageUp => {
                    self.cpp_checks.scroll = self.cpp_checks.scroll.saturating_sub(10)
                }
                KeyCode::PageDown => {
                    self.cpp_checks.scroll = self.cpp_checks.scroll.saturating_add(10).min(max)
                }
                _ => {}
            }
            return true;
        }
        false
    }

    pub(crate) fn draw_cpp_check(&self, frame: &mut Frame) {
        if let Some(report) = &self.cpp_checks.report {
            let screen = frame.area();
            let area = Rect::new(
                screen.x,
                screen.y,
                screen.width,
                screen.height.saturating_sub(2),
            );
            frame.render_widget(Clear, area);
            frame.render_widget(
                Paragraph::new(report.as_str())
                    .scroll((self.cpp_checks.scroll, 0))
                    .block(Block::bordered().title(" C++ check · ↑/↓ PgUp/PgDn · Esc: close ")),
                area,
            );
        }
    }
}
