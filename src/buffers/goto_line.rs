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

//! Go to line (M-g g or M-g M-g, as in Emacs).

use super::menu::LIGHT_BLUE;
use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Padding, Paragraph},
};

#[derive(Default)]
pub(crate) struct GotoLine {
    // M-g was pressed.
    prefix: bool,

    // The typed line number while the dialog is open.
    input: Option<String>,
}

impl GotoLine {
    pub(crate) fn after_prefix(&self) -> bool {
        self.prefix
    }

    pub(crate) fn is_open(&self) -> bool {
        self.input.is_some()
    }

    // True while M-g waits for its next key or the dialog is open.
    pub(crate) fn is_busy(&self) -> bool {
        self.prefix || self.is_open()
    }
}

impl App {
    // Keys of M-g g and of the dialog. Returns true when the key was used.
    pub(crate) fn handle_goto_line_key(&mut self, key: KeyEvent) -> bool {
        let alt = key.modifiers == KeyModifiers::ALT;
        if let Some(input) = &mut self.goto_line.input {
            match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() && key.modifiers.is_empty() => input.push(c),
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => self.go_to_typed_line(),
                KeyCode::Esc => self.goto_line.input = None,
                KeyCode::Char('g') if key.modifiers == KeyModifiers::CONTROL => {
                    self.goto_line.input = None;
                }

                // The dialog keeps every other key until it closes.
                _ => {}
            }
            return true;
        }
        if std::mem::take(&mut self.goto_line.prefix) {
            // M-g g and M-g M-g open the dialog; other keys after M-g do nothing.
            if key.code == KeyCode::Char('g') && (key.modifiers.is_empty() || alt) {
                self.completion.close();
                self.goto_line.input = Some(String::new());
            }
            return true;
        }

        // C-c chords belong to completion and the checks.
        if key.code == KeyCode::Char('g') && alt && !self.ctrl_c_wait_flag {
            self.goto_line.prefix = true;
            return true;
        }
        false
    }

    fn go_to_typed_line(&mut self) {
        let typed = self.goto_line.input.take().unwrap_or_default();

        // Nothing typed closes the dialog.
        let Ok(number) = typed.parse::<usize>() else {
            return;
        };

        // Like Emacs, 0 goes to the first line and numbers past the end to the last one.
        let code = self.buffer.editor.code_ref();
        let index = number
            .saturating_sub(1)
            .min(code.len_lines().saturating_sub(1));
        let start = code.line_to_char(index);
        self.buffer.editor.set_selection(None);
        self.buffer.editor.set_cursor(start);
        self.buffer.editor.focus(&self.editor_area);
    }

    // The dialog, in the middle of the screen.
    pub(crate) fn draw_goto_line(&self, frame: &mut Frame) {
        let Some(input) = &self.goto_line.input else {
            return;
        };
        let lines = self.buffer.editor.code_ref().len_lines();
        let base = Style::default().fg(LIGHT_BLUE);
        let text = vec![
            Line::from(vec![
                Span::styled(format!("Line (1-{lines}): "), base),
                Span::styled(format!("{input}█"), base.add_modifier(Modifier::BOLD)),
            ]),
            Line::styled("Enter: go  Esc: cancel", base.add_modifier(Modifier::DIM)),
        ];

        // Borders and one column of padding on each side take four columns.
        let area = frame.area();
        let width = text.iter().map(Line::width).max().unwrap_or(0) as u16 + 4;
        let width = width.max(TITLE.len() as u16 + 2).min(area.width);
        let height = (text.len() as u16 + 2).min(area.height);
        let popup = Rect::new(
            area.x + (area.width - width) / 2,
            area.y + (area.height - height) / 2,
            width,
            height,
        );
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(base)
            .title(TITLE)
            .padding(Padding::horizontal(1));
        frame.render_widget(Clear, popup);
        frame.render_widget(Paragraph::new(text).block(block), popup);
    }
}

const TITLE: &str = " Go to line ";

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn press(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
        app.handle_key(KeyEvent::new(code, modifiers));
    }

    fn goto(app: &mut App, typed: &str) {
        press(app, KeyCode::Char('g'), KeyModifiers::ALT);
        press(app, KeyCode::Char('g'), KeyModifiers::NONE);
        for c in typed.chars() {
            press(app, KeyCode::Char(c), KeyModifiers::NONE);
        }
        press(app, KeyCode::Enter, KeyModifiers::NONE);
    }

    fn app_with(text: &str) -> App {
        let mut app = App::default();
        app.editor_area = Rect::new(1, 1, 78, 22);
        app.buffer.editor.set_content(text);
        app
    }

    #[test]
    fn goes_to_the_start_of_the_line_and_clamps_like_emacs() {
        let mut app = app_with("one\ntwo\nthree\nfour");
        goto(&mut app, "3");
        assert_eq!(app.buffer.editor.get_cursor(), 8);
        assert!(!app.goto_line.is_open());
        goto(&mut app, "99");
        assert_eq!(app.buffer.editor.get_cursor(), 14, "last line");
        goto(&mut app, "0");
        assert_eq!(app.buffer.editor.get_cursor(), 0, "first line");
        assert_eq!(app.buffer.editor.get_content(), "one\ntwo\nthree\nfour");
    }

    #[test]
    fn dialog_takes_only_digits_and_can_be_cancelled() {
        let mut app = app_with("a\nb\nc");
        app.buffer.editor.set_cursor(2);

        // M-g M-g works too; letters are ignored and Backspace edits the number.
        press(&mut app, KeyCode::Char('g'), KeyModifiers::ALT);
        assert_eq!(app.buffer_command, "M-g");
        press(&mut app, KeyCode::Char('g'), KeyModifiers::ALT);
        assert_eq!(app.buffer_command, "M-g M-g");
        assert!(app.goto_line.is_open());
        for code in [KeyCode::Char('x'), KeyCode::Char('3'), KeyCode::Backspace] {
            press(&mut app, code, KeyModifiers::NONE);
        }
        press(&mut app, KeyCode::Esc, KeyModifiers::NONE);
        assert!(!app.goto_line.is_open());
        assert_eq!(app.buffer.editor.get_cursor(), 2);
        assert_eq!(app.buffer.editor.get_content(), "a\nb\nc");

        // Enter with nothing typed only closes; M-g with another key does nothing.
        goto(&mut app, "");
        assert_eq!(app.buffer.editor.get_cursor(), 2);
        press(&mut app, KeyCode::Char('g'), KeyModifiers::ALT);
        press(&mut app, KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!app.goto_line.is_busy());
        assert_eq!(app.buffer.editor.get_content(), "a\nb\nc");
    }

    #[test]
    fn m_g_ends_an_incremental_search_first() {
        let mut app = app_with("a\nb\nc");
        press(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        press(&mut app, KeyCode::Char('b'), KeyModifiers::NONE);
        goto(&mut app, "3");
        assert!(app.search.prompt().is_none());
        assert_eq!(app.buffer.editor.get_cursor(), 4);

        // While the dialog is open, C-s does not start a search.
        press(&mut app, KeyCode::Char('g'), KeyModifiers::ALT);
        press(&mut app, KeyCode::Char('g'), KeyModifiers::NONE);
        press(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert!(app.search.prompt().is_none());
        assert!(app.goto_line.is_open());
    }

    #[test]
    fn dialog_is_drawn_in_the_middle() {
        let mut app = app_with("1\n2\n3\n4");
        press(&mut app, KeyCode::Char('g'), KeyModifiers::ALT);
        press(&mut app, KeyCode::Char('g'), KeyModifiers::NONE);
        press(&mut app, KeyCode::Char('2'), KeyModifiers::NONE);
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let screen = terminal.backend().buffer();
        let rows: Vec<String> = (0..24)
            .map(|y| (0..80).map(|x| screen[(x, y)].symbol()).collect())
            .collect();
        assert!(rows[10].contains("╭ Go to line"), "{}", rows[10]);
        assert!(rows[11].contains("│ Line (1-4): 2█"), "{}", rows[11]);
        assert!(rows[12].contains("Enter: go  Esc: cancel"));
    }
}
