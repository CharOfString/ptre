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

// Quick fixes (C-c a): ask the language server how to fix the problem under the cursor,
// then apply the fix, or let the user pick one when there are several.

use super::menu::LIGHT_BLUE;
use crate::{
    app::App,
    lsp::{Fix, Fixes},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, List, ListItem, ListState},
};
use std::ops::Range;

const TITLE: &str = " Quick fix ";

#[derive(Default)]
pub(crate) struct QuickFix {
    waiting: bool,
    // Avaliable autofixes that user may select.
    choices: Vec<Fix>,

    // Where shall the fix apply to.
    text: String,
    selected: usize,
}

impl QuickFix {
    // True while the server's answer is expected.
    pub(crate) fn is_waiting(&self) -> bool {
        self.waiting
    }

    pub(crate) fn is_open(&self) -> bool {
        !self.choices.is_empty()
    }

    // The server stopped, no answer will come.
    pub(crate) fn stop_waiting(&mut self) {
        self.waiting = false;
    }
}

impl App {
    pub(crate) fn request_fix(&mut self) {
        self.completion.close();
        let text = self.buffer.editor.get_content();
        let cursor = self.buffer.editor.get_cursor();
        if self.completion.request_fixes(&text, cursor) {
            self.quick_fix.waiting = true;
            self.status_bar_text = "Looking for fixes...".into();
        } else {
            self.status_bar_text = "No language server problem at the cursor".into();
        }
    }

    // Apply the only fix, or open the chooser for several.
    pub(crate) fn receive_fixes(&mut self, answer: Fixes) {
        if !std::mem::take(&mut self.quick_fix.waiting) {
            return;
        }

        match answer.fixes.as_slice() {
            [] => self.status_bar_text = "No fixes available".into(),
            [fix] => self.apply_fix(fix, &answer.text),
            _ => {
                self.completion.close();
                self.quick_fix.choices = answer.fixes;
                self.quick_fix.text = answer.text;
                self.quick_fix.selected = 0;
                self.status_bar_text = "Choose a fix => Enter: apply  Esc: cancel".into();
            }
        }
    }

    // Replace the fix's ranges as one edit.
    fn apply_fix(&mut self, fix: &Fix, text: &str) {
        let editor = &mut self.buffer.editor;
        if editor.get_content() != text {
            self.status_bar_text = "The text changed => press C-c a again".into();
            return;
        }
        let cursor = editor.get_cursor();
        let moved = moved_cursor(cursor, &fix.edits);
        let code = editor.code_mut();
        code.tx();
        code.set_state_before(cursor, None);

        // From the end backwards, so earlier ranges keep their offsets.
        let mut edits: Vec<_> = fix.edits.iter().collect();
        edits.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
        for (range, new_text) in edits {
            code.remove(range.start, range.end);
            code.insert(range.start, new_text);
        }
        code.set_state_after(moved, None);
        code.commit();
        editor.set_cursor(moved);
        editor.set_selection(None);
        editor.reset_highlight_cache();
        editor.focus(&self.editor_area);
        self.status_bar_text = format!("Fixed: {}", fix.title);
    }

    // Keys of the fix chooser. Returns true when the key was used.
    pub(crate) fn handle_fix_key(&mut self, key: KeyEvent) -> bool {
        if !self.quick_fix.is_open() {
            return false;
        }
        let control = key.modifiers == KeyModifiers::CONTROL;
        let count = self.quick_fix.choices.len();
        let selected = &mut self.quick_fix.selected;
        match key.code {
            KeyCode::Up => *selected = (*selected + count - 1) % count,
            KeyCode::Char('p') if control => *selected = (*selected + count - 1) % count,
            KeyCode::Down => *selected = (*selected + 1) % count,
            KeyCode::Char('n') if control => *selected = (*selected + 1) % count,
            KeyCode::Enter | KeyCode::Tab => {
                let fix = self.quick_fix.choices.swap_remove(*selected);
                let text = std::mem::take(&mut self.quick_fix.text);
                self.quick_fix.choices.clear();
                self.apply_fix(&fix, &text);
            }
            KeyCode::Esc => self.quick_fix.choices.clear(),
            KeyCode::Char('g') if control => self.quick_fix.choices.clear(),
            // Any other key closes the chooser and still reaches the editor.
            _ => {
                self.quick_fix.choices.clear();
                return false;
            }
        }
        true
    }

    // The chooser, drawn under the cursor like the completion list.
    pub(crate) fn draw_fix_chooser(&self, frame: &mut Frame) {
        let quick_fix = &self.quick_fix;
        if !quick_fix.is_open() {
            return;
        }
        let Some((x, y)) = self.buffer.editor.get_visible_cursor(&self.editor_area) else {
            return;
        };
        let base = Style::default().fg(LIGHT_BLUE);
        let rows: Vec<ListItem> = quick_fix
            .choices
            .iter()
            .enumerate()
            .map(|(row, fix)| {
                let style = if row == quick_fix.selected {
                    base.add_modifier(Modifier::BOLD)
                } else {
                    base
                };
                ListItem::new(Line::from(Span::styled(fix.title.as_str(), style)))
            })
            .collect();

        // Two borders plus the two-column selection marker, and room for the title.
        let area = frame.area();
        let width = rows.iter().map(ListItem::width).max().unwrap_or(0) as u16 + 4;
        let width = width.max(TITLE.len() as u16 + 2).min(area.width);
        let height = rows.len().min(8) as u16 + 2;
        let x = x.saturating_sub(3).min(area.right().saturating_sub(width));
        let y = if y + 1 + height <= area.bottom() {
            y + 1
        } else {
            y.saturating_sub(height)
        };
        let popup = Rect::new(x, y, width, height).intersection(area);
        let list = List::new(rows)
            .style(base)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(base)
                    .title(TITLE),
            )
            .highlight_symbol("❃ ");
        let mut state = ListState::default().with_selected(Some(quick_fix.selected));
        frame.render_widget(Clear, popup);
        frame.render_stateful_widget(list, popup, &mut state);
    }
}

// Where cursor ends up afte edits: shifted by the edits before it, or at the end of
// the new text of an edit that replaced it.
fn moved_cursor(cursor: usize, edits: &[(Range<usize>, String)]) -> usize {
    let mut sorted: Vec<_> = edits.iter().collect();
    sorted.sort_by_key(|(range, _)| range.start);
    let mut shift = 0isize;
    for (range, new_text) in sorted {
        let new_length = new_text.chars().count() as isize;
        if range.end <= cursor {
            shift += new_length - range.len() as isize;
        } else if range.start < cursor {
            return (range.start as isize + shift + new_length) as usize;
        } else {
            break;
        }
    }
    (cursor as isize + shift) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn press(app: &mut App, code: KeyCode) {
        app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
    }

    fn fix(title: &str, edits: &[(Range<usize>, &str)]) -> Fix {
        let edits = edits.iter().map(|(r, t)| (r.clone(), (*t).to_owned()));
        Fix {
            title: title.into(),
            edits: edits.collect(),
        }
    }

    fn waiting_app(text: &str, cursor: usize) -> App {
        let mut app = App::default();
        app.buffer.editor.set_content(text);
        app.buffer.editor.set_cursor(cursor);
        app.quick_fix.waiting = true;
        app
    }

    #[test]
    fn cursor_follows_the_edits() {
        let edits = |list: &[(Range<usize>, &str)]| fix("", list).edits;
        assert_eq!(moved_cursor(10, &edits(&[(2..4, "abcd")])), 12);
        assert_eq!(moved_cursor(3, &edits(&[(2..6, "xy")])), 4, "inside");
        assert_eq!(moved_cursor(2, &edits(&[(2..6, "xy")])), 2, "at the start");
        assert_eq!(
            moved_cursor(5, &edits(&[(5..5, ";")])),
            6,
            "insert at cursor"
        );
        assert_eq!(moved_cursor(5, &edits(&[(8..9, ""), (0..1, "")])), 4);
    }

    #[test]
    fn a_single_fix_is_applied_as_one_undoable_edit() {
        let text = "int value_one;\nint x = value_on\n";
        let mut app = waiting_app(text, 25);
        app.receive_fixes(Fixes {
            fixes: vec![fix(
                "change to value_one",
                &[(23..31, "value_one"), (31..31, ";")],
            )],
            text: text.into(),
        });
        assert_eq!(
            app.buffer.editor.get_content(),
            "int value_one;\nint x = value_one;\n"
        );
        assert_eq!(app.buffer.editor.get_cursor(), 32);
        assert_eq!(app.status_bar_text, "Fixed: change to value_one");
        app.handle_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        assert_eq!(app.buffer.editor.get_content(), text);
    }

    #[test]
    fn fixes_for_old_text_or_without_request_are_not_applied() {
        let mut app = waiting_app("abc", 0);
        let answer = || Fixes {
            fixes: vec![fix("x", &[(0..1, "x")])],
            text: "abd".into(),
        };
        app.receive_fixes(answer());
        assert_eq!(app.buffer.editor.get_content(), "abc");
        assert!(app.status_bar_text.contains("changed"));
        // Without a request waiting, an answer is ignored.
        app.status_bar_text.clear();
        app.receive_fixes(answer());
        assert!(app.status_bar_text.is_empty());
        let mut app = waiting_app("abc", 0);
        app.receive_fixes(Fixes::default());
        assert_eq!(app.status_bar_text, "No fixes available");
    }

    #[test]
    fn several_fixes_open_a_chooser() {
        let mut app = waiting_app("abc", 1);
        app.receive_fixes(Fixes {
            fixes: vec![fix("first", &[(0..1, "1")]), fix("second", &[(2..3, "3")])],
            text: "abc".into(),
        });
        assert!(app.quick_fix.is_open());
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let screen = terminal.backend().buffer();
        let text: String = (0..24)
            .flat_map(|y| (0..80).map(move |x| (x, y)))
            .map(|cell| screen[cell].symbol())
            .collect();
        assert!(text.contains("Quick fix"));
        assert!(text.contains("❃ first"));
        assert!(text.contains("  second"));

        press(&mut app, KeyCode::Down);
        press(&mut app, KeyCode::Enter);
        assert!(!app.quick_fix.is_open());
        assert_eq!(app.buffer.editor.get_content(), "ab3");

        // Esc closes without changes; other keys close and still reach the editor.
        let mut app = waiting_app("abc", 3);
        let answer = || Fixes {
            fixes: vec![fix("a", &[(0..1, "A")]), fix("b", &[(1..2, "B")])],
            text: "abc".into(),
        };
        app.receive_fixes(answer());
        press(&mut app, KeyCode::Esc);
        assert!(!app.quick_fix.is_open());
        assert_eq!(app.buffer.editor.get_content(), "abc");
        app.quick_fix.waiting = true;
        app.receive_fixes(answer());
        press(&mut app, KeyCode::Char('!'));
        assert!(!app.quick_fix.is_open());
        assert_eq!(app.buffer.editor.get_content(), "abc!");
    }
}
