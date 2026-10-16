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
        self.menu.draw_bar(frame, menu_area, self.is_c_or_cpp());

        let editor_area = if self.terminal.visible {
            let [editor, terminal] = super::terminal::split(editor_area);
            if let Err(err) = self.terminal.draw(frame, terminal) {
                self.status_bar_text = format!("Terminal resize failed: {err}");
            }
            editor
        } else {
            editor_area
        };

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
        // While the paste scroll runs, draw the editor parts from its current line.
        let offset_y = self.buffer.editor.get_offset_y();
        if let Some(line) = self.paste_scroll_offset() {
            self.buffer.editor.set_offset_y(line);
        }
        frame.render_widget(border, editor_area);
        frame.render_widget(&self.buffer.editor, self.editor_area);
        self.draw_diagnostics(frame);
        super::editor_status::draw(
            &self.buffer.editor,
            self.diagnostic_counts(),
            self.nerd_font,
            frame,
            buffer_status_area,
            editor_area,
        );

        self.draw_completion(frame);
        self.draw_fix_chooser(frame);
        self.draw_diagnostic_popup(frame);
        if !self.terminal.focused
            && !self.cpp_check_modal()
            && self.save_path_input.is_none()
            && !self.search.reads_input()
            && !self.goto_line.is_open()
            && !self.overwrite_confirm
            && self.menu.active.is_none()
            && let Some((x, y)) = self.buffer.editor.get_visible_cursor(&self.editor_area)
        {
            frame.set_cursor_position((x, y));
        }
        self.buffer.editor.set_offset_y(offset_y);

        // status_bar_text bar.
        let status_bar_text_text = if let Some(input) = &self.cpp_checks.path_input {
            format!(
                " Cpplint path: {input}█ ※ Enter: enable Esc: cancel ※ {}",
                self.status_bar_text
            )
        } else if let Some(prompt) = self.search.prompt() {
            format!(" {prompt}")
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
            // A focused terminal takes all keys, so show its keys instead.
            let hints = if self.terminal.focused {
                "C-x o: focus ※  M-v/C-v: scroll"
            } else {
                "F10: menu ※  C-c t t: terminal"
            };

            if self.status_bar_text.is_empty() {
                if self.buffer_command.is_empty() {
                    format!(" {name} ※  {hints}")
                } else {
                    format!(
                        " {name} ※  {hints} ※  Buffer command: {}",
                        self.buffer_command
                    )
                }
            } else {
                if self.buffer_command.is_empty() {
                    format!(" {name} ※  {hints} ※  {}", self.status_bar_text)
                } else {
                    format!(
                        " {name} ※  {hints} ※  Buffer command: {}  ※ {}",
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
            self.nerd_font,
            self.is_c_or_cpp().then_some(&self.cpp_checks.settings),
        );

        self.draw_goto_line(frame);
        self.draw_cpp_check(frame);

        if self.overwrite_confirm {
            super::dialog::draw_overwrite(frame, self.buffer.path.as_deref());
        }
    }
}
