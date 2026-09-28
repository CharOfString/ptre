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
    widgets::{Block, BorderType, Borders, Paragraph},
};

impl App {
    pub(crate) fn draw(&mut self, frame: &mut Frame) {
        let [menu_area, editor_area, status_bar_text_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .areas(frame.area());
        self.menu.draw_bar(frame, menu_area);

        // Editor buffer.
        let name = self
            .buffer
            .path
            .as_ref()
            .and_then(|path| path.file_name())
            .map_or_else(|| "[No Name]".into(), |name| name.to_string_lossy());
        let mut title = format!("─ {name}");

        // Buffer indicators.
        if self.buffer.is_dirty() {
            title.push_str(" · DIRTY BUFFER");
        }
        if self.buffer.obsolete {
            title.push_str(" · OBSOLETE FILE");
        }
        title.push(' ');
        let border = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White))
            .title(title);
        let [content_area, buffer_status_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(2)])
                .areas(border.inner(editor_area));
        self.editor_area = content_area;
        frame.render_widget(border, editor_area);
        frame.render_widget(&self.buffer.editor, self.editor_area);
        super::editor_status::draw(&self.buffer.editor, frame, buffer_status_area, editor_area);
        self.draw_completion(frame);
        if !self.cpp_check_modal()
            && self.save_path_input.is_none()
            && !self.overwrite_confirm
            && self.menu.active.is_none()
            && let Some((x, y)) = self.buffer.editor.get_visible_cursor(&self.editor_area)
        {
            frame.set_cursor_position((x, y));
        }

        // status_bar_text bar.
        let status_bar_text_text = if let Some(input) = &self.cpp_checks.path_input {
            format!(
                " Cpplint path: {input}█ ※ Enter: enable Esc: cancel ※ {}",
                self.status_bar_text
            )
        } else if let Some(path_input) = &self.save_path_input {
            format!(
                " Save as: {path_input}█  ※  Enter: save  Esc: cancel  ※  {}",
                self.status_bar_text
            )
        } else {
            let name = self
                .buffer
                .path
                .as_ref()
                .map_or_else(|| "[No Name]".to_owned(), |path| path.display().to_string());

            if self.status_bar_text.is_empty() {
                if self.buffer_command.is_empty() {
                    format!(" {name} ※  F10: menu")
                } else {
                    format!(
                        " {name} ※  F10: menu ※  Buffer command: {}",
                        self.buffer_command
                    )
                }
            } else {
                if self.buffer_command.is_empty() {
                    format!(" {name} ※  F10: menu ※  {}", self.status_bar_text)
                } else {
                    format!(
                        " {name} ※  F10: menu ※  Buffer command: {}  ※ {}",
                        self.buffer_command, self.status_bar_text
                    )
                }
            }
        };
        let status_bar_text = Paragraph::new(status_bar_text_text)
            .style(Style::default().fg(Color::Green))
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::Green)),
            );

        // Init render
        frame.render_widget(status_bar_text, status_bar_text_area);

        self.menu.draw_popup(
            frame,
            self.completion.auto_enabled(),
            self.is_cpp().then_some(&self.cpp_checks.settings),
        );

        self.draw_cpp_check(frame);

        if self.overwrite_confirm {
            super::dialog::draw_overwrite(frame, self.buffer.path.as_deref());
        }
    }
}
