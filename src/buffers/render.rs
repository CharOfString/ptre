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

use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Paragraph},
};

impl App {
    pub(crate) fn draw(&self, frame: &mut Frame) {
        // We have buffer on top + status_bar_text bar.
        let [editor_area, status_bar_text_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());

        // Editor buffer.
        let editor =
            Paragraph::new(self.buffer.text.as_str()).block(Block::bordered().title(" Pointer "));
        frame.render_widget(editor, editor_area);

        // status_bar_text bar.
        let status_bar_text_text = if let Some(path_input) = &self.save_path_input {
            format!(
                " Save as: {path_input}█  |  Enter: save  Esc: cancel  |  {}",
                self.status_bar_text
            )
        } else {
            let name = self
                .buffer
                .path
                .as_ref()
                .map_or_else(|| "[No Name]".to_owned(), |path| path.display().to_string());
            format!(
                " {name} | {} chars | C-x C-s: save | C-x C-c: quit | {}",
                self.buffer.text.chars().count(),
                self.status_bar_text
            )
        };
        let status_bar_text = Paragraph::new(status_bar_text_text)
            .style(Style::default().fg(Color::Black).bg(Color::Cyan));
        // Init render
        frame.render_widget(status_bar_text, status_bar_text_area);
    }
}
