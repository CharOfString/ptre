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

//! File tabs above the editor: C-x Right and C-x Left switch tabs, C-x k closes one.

use super::{editor::Buffer, menu::LIGHT_BLUE};
use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::{
    io,
    path::{Path, PathBuf},
};

// Colors of the other tabs and of the unsaved mark, as in the confirm dialog.
const MUTED: Color = Color::Rgb(140, 138, 145);
const AMBER: Color = Color::Rgb(232, 181, 106);

// The active tab is App::buffer. The other tabs wait here, in screen order.
#[derive(Default)]
pub(crate) struct Tabs {
    left: Vec<Buffer>,
    right: Vec<Buffer>,
    // True while the dialog asks whether to close a tab with unsaved changes.
    pub(crate) close_confirm: bool,
}

fn name(buffer: &Buffer) -> String {
    let name = buffer.path.as_deref().and_then(Path::file_name);
    name.map_or_else(|| "[No Name]".into(), |name| name.to_string_lossy().into())
}

// Single tab.
fn tab(buffer: &Buffer, active: bool) -> Vec<Span<'static>> {
    let (marker, style) = if active {
        (
            "❃ ",
            Style::default().fg(LIGHT_BLUE).add_modifier(Modifier::BOLD),
        )
    } else {
        ("  ", Style::default().fg(MUTED))
    };

    let mut spans = vec![
        Span::styled(marker, style),
        Span::styled(name(buffer), style),
    ];

    if buffer.is_dirty() {
        spans.push(Span::styled(" ●", Style::default().fg(AMBER)));
    }
    spans.push(Span::raw("  "));
    spans
}

impl App {
    // All tabs in screen order.
    fn tab_buffers(&self) -> impl Iterator<Item = &Buffer> {
        let tabs = &self.tabs;
        tabs.left.iter().chain([&self.buffer]).chain(&tabs.right)
    }

    pub(crate) fn has_other_tabs(&self) -> bool {
        !self.tabs.left.is_empty() || !self.tabs.right.is_empty()
    }

    // Make the tab at `index` active. An index past the last tab goes round to the first.
    fn select_tab(&mut self, index: usize) {
        let mut all = std::mem::take(&mut self.tabs.left);
        all.push(std::mem::take(&mut self.buffer));
        all.append(&mut self.tabs.right);
        let index = index % all.len();
        self.tabs.right = all.split_off(index + 1);
        self.buffer = all.pop().unwrap_or_default();
        self.tabs.left = all;
        self.paste_scroll = None;
        self.buffer.check_disk();
    }

    // Show `path` in a tab: its own tab when it is open already, else a new one.
    pub(crate) fn open_tab(&mut self, path: PathBuf) -> io::Result<()> {
        let target = std::path::absolute(&path)?;
        let index = self.tab_buffers().position(|buffer| {
            let path = buffer.path.as_deref();
            path.and_then(|path| std::path::absolute(path).ok())
                .as_ref()
                == Some(&target)
        });
        if let Some(index) = index {
            self.select_tab(index);
            return Ok(());
        }
        let mut buffer = Buffer::default();
        buffer.open(path)?;
        if self.buffer.path.is_none() && self.buffer.editor.get_content().is_empty() {
            // The file takes the place of an empty unnamed tab.
            self.buffer = buffer;
        } else {
            // The new tab goes right after the active one.
            let old = std::mem::replace(&mut self.buffer, buffer);
            self.tabs.left.push(old);
        }
        self.paste_scroll = None;
        Ok(())
    }

    // Close the active tab. The tab on its right takes its place, else the one on its left.
    fn close_tab(&mut self) {
        let next = if self.tabs.right.is_empty() {
            self.tabs.left.pop()
        } else {
            Some(self.tabs.right.remove(0))
        };
        // Closing the last tab leaves an empty one.
        self.buffer = next.unwrap_or_default();
        self.paste_scroll = None;
        self.buffer.check_disk();
    }

    // Keys of the close dialog, C-x Right, C-x Left and C-x k. Returns true when used.
    pub(crate) fn handle_tab_key(&mut self, key: KeyEvent) -> bool {
        if self.tabs.close_confirm {
            let plain = key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT;
            match key.code {
                KeyCode::Char('y' | 'Y') | KeyCode::Enter if plain => {
                    self.tabs.close_confirm = false;
                    self.close_tab();
                }
                KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                    self.tabs.close_confirm = false;
                    self.status_bar_text = "Close cancelled".into();
                }
                _ => {}
            }
            return true;
        }
        if !self.ctrl_x_wait_flag || !key.modifiers.is_empty() {
            return false;
        }
        let active = self.tabs.left.len();
        let count = active + 1 + self.tabs.right.len();
        match key.code {
            KeyCode::Right => self.select_tab(active + 1),
            KeyCode::Left => self.select_tab(active + count - 1),
            // Unsaved changes would be lost, so ask first.
            KeyCode::Char('k') if self.buffer.is_dirty() => self.tabs.close_confirm = true,
            KeyCode::Char('k') => self.close_tab(),
            _ => return false,
        }
        self.ctrl_x_wait_flag = false;
        true
    }

    pub(crate) fn draw_tabs(&self, frame: &mut Frame, area: Rect) {
        let area = area.inner(Margin::new(1, 0));
        let active = self.tabs.left.len();
        let mut tabs: Vec<_> = self
            .tab_buffers()
            .enumerate()
            .map(|(index, buffer)| tab(buffer, index == active))
            .collect();

        let fits = |first: usize, last: usize| {
            let mut width: usize = tabs[first..=last].iter().flatten().map(Span::width).sum();
            if first > 0 && first == active {
                width += 2;
            }

            if last + 1 == tabs.len() {
                width -= 2;
            }
            width <= usize::from(area.width)
        };

        let mut first = 0;
        while first < active && !fits(first, active) {
            first += 1;
        }

        let mut last = active;
        while last + 1 < tabs.len() && fits(first, last + 1) {
            last += 1;
        }

        let arrow = |text| Span::styled(text, Style::default().fg(MUTED));
        if last + 1 < tabs.len() {
            *tabs[last].last_mut().unwrap() = arrow(" →");
        }

        if first > 0 && first == active {
            tabs[first].insert(0, arrow("← "));
        } else if first > 0 {
            tabs[first][0] = arrow("← ");
        }

        let spans: Vec<_> = tabs.drain(first..=last).flatten().collect();
        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }
}

#[cfg(test)]
mod tests;
