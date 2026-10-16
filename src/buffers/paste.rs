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

//! Text pasted by the terminal emulator (bracketed paste).

use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_code_editor::actions::InsertText;
use std::time::{Duration, Instant};

// The scroll to the cursor ends well before 300ms, even with a slow frame.
const SCROLL_TIME: Duration = Duration::from_millis(250);

// After a paste, the view moves from where it was to the cursor.
pub(crate) struct Scroll {
    from: usize,
    start: Instant,
}

// The first visible line at a point of the scroll; progress goes from 0 to 1.
fn scroll_offset(from: usize, to: usize, progress: f64) -> usize {
    // Ease out: move fast first, then slow down near the cursor.
    let eased = 1.0 - (1.0 - progress.min(1.0)).powi(3);
    (from as f64 + (to as f64 - from as f64) * eased).round() as usize
}

impl App {
    // The terminal emulator sends the whole pasted text in one event.
    pub(crate) fn handle_paste(&mut self, text: String) {
        // The shell in the terminal pane gets the text unchanged.
        if self.terminal.focused && !self.prompt_open() {
            if let Err(err) = self.terminal.paste(&text) {
                self.status_bar_text = format!("Terminal input failed: {err}");
            }
            return;
        }

        // Prompts, the menu and key chords read single keys, so type the text into them.
        // Line breaks and other control characters are left out.
        if self.keys_go_elsewhere() {
            for c in text.chars().filter(|c| !c.is_control()) {
                self.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
            }
            return;
        }

        // Insert the text as it is, in one edit. Typing it key by key would
        // indent each new line again and redraw the screen after each key.
        // Terminals send line breaks as "\r", but the buffer uses "\n".
        let text = text.replace("\r\n", "\n").replace('\r', "\n");
        let from = self.buffer.editor.get_offset_y();
        self.completion.close();
        self.buffer.editor.apply(InsertText { text });
        self.buffer.editor.focus(&self.editor_area);

        // The view is already at the cursor; only the drawing scrolls there.
        if self.buffer.editor.get_offset_y() != from {
            self.paste_scroll = Some(Scroll {
                from,
                start: Instant::now(),
            });
        }
    }

    // The first line to draw while the paste scroll runs, or None when it is over.
    pub(crate) fn paste_scroll_offset(&mut self) -> Option<usize> {
        let scroll = self.paste_scroll.as_ref()?;
        let progress = scroll.start.elapsed().as_secs_f64() / SCROLL_TIME.as_secs_f64();
        if progress >= 1.0 {
            self.paste_scroll = None;
            return None;
        }
        // Aim at the real view, which keys may move while the scroll runs.
        let to = self.buffer.editor.get_offset_y();
        Some(scroll_offset(scroll.from, to, progress))
    }

    // True when typed keys do not go straight into the text.
    fn keys_go_elsewhere(&self) -> bool {
        self.prompt_open()
            || self.search.is_active()
            || self.goto_line.is_busy()
            || self.quick_fix.is_open()
            || self.ctrl_x_wait_flag
            || self.ctrl_c_wait_flag
            || self.ctrl_c_t_wait_flag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paste_keeps_indentation_and_undoes_in_one_step() {
        let mut app = App::default();
        app.buffer.editor.set_content("fn main() {\n}\n");
        app.buffer.editor.set_cursor(12);
        app.handle_paste("    if ok {\r        run();\r    }\r\n".into());
        assert_eq!(
            app.buffer.editor.get_content(),
            "fn main() {\n    if ok {\n        run();\n    }\n}\n"
        );
        assert_eq!(app.buffer.editor.get_cursor(), 45);

        app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE));
        assert_eq!(app.buffer.editor.get_content(), "fn main() {\n}\n");
    }

    #[test]
    fn scroll_starts_at_the_old_view_and_ends_at_the_cursor() {
        assert_eq!(scroll_offset(0, 85, 0.0), 0);
        // Ease out: past half way at half time.
        let middle = scroll_offset(0, 85, 0.5);
        assert!(42 < middle && middle < 85, "{middle}");
        assert_eq!(scroll_offset(0, 85, 1.0), 85);
        assert_eq!(scroll_offset(85, 10, 2.0), 10);
    }

    #[test]
    fn paste_scroll_is_only_drawn_and_ends_within_300ms() {
        use ratatui::{Terminal, backend::TestBackend};
        let screen = |terminal: &Terminal<TestBackend>| -> String {
            let buffer = terminal.backend().buffer();
            buffer.content.iter().map(|cell| cell.symbol()).collect()
        };
        let mut app = App::default();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        // Draw once so the paste knows the size of the editor.
        terminal.draw(|frame| app.draw(frame)).unwrap();
        app.handle_paste((0..100).map(|n| format!("line {n}\r")).collect());
        let end = app.buffer.editor.get_offset_y();
        assert!(end > 0 && app.paste_scroll.is_some());

        // Drawing does not move the real view away from the cursor.
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert_eq!(app.buffer.editor.get_offset_y(), end);

        // When the time is over, the view rests at the cursor.
        assert!(SCROLL_TIME < Duration::from_millis(300));
        app.paste_scroll.as_mut().unwrap().start -= SCROLL_TIME;
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert!(app.paste_scroll.is_none());
        assert!(screen(&terminal).contains("line 99"));
        assert!(!screen(&terminal).contains("line 0 "));
    }

    #[test]
    fn prompts_get_the_text_without_line_breaks() {
        let mut app = App::default();
        app.save_path_input = Some(String::new());
        app.handle_paste("notes.txt\r".into());
        assert_eq!(app.save_path_input.as_deref(), Some("notes.txt"));
        assert_eq!(app.buffer.editor.get_content(), "");
    }

    #[test]
    fn focused_terminal_gets_the_text() {
        let mut app = App::default();
        app.terminal.visible = true;
        app.terminal.focused = true;
        app.handle_paste("ls\r".into());
        assert_eq!(app.buffer.editor.get_content(), "");
    }
}
