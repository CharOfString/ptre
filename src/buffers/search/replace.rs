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

// Query replace (M-%).

use super::{Mode, find_all};
use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::ops::Range;

pub(super) struct Replacing {
    from: String,
    to: String,

    // The match.
    current: Range<usize>,
    count: usize,

    // The cursor before the first replacement, restored by undo.
    origin: usize,

    // All replacements form one undo step, opened at the first one.
    open: bool,
}

impl Replacing {
    pub(super) fn prompt(&self) -> String {
        format!(
            "Query replacing {} with {}: (y, n, !, ., q)",
            self.from, self.to
        )
    }
}

impl App {
    pub(super) fn start_replace(&mut self) {
        self.search.mode = Some(Mode::ReplaceFrom(String::new()));
    }

    // Typing the text to replace and its replacement.
    pub(super) fn replace_prompt_key(&mut self, key: KeyEvent) -> bool {
        let control = key.modifiers == KeyModifiers::CONTROL;
        let plain = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        let Some(Mode::ReplaceFrom(input) | Mode::ReplaceTo(_, input)) = &mut self.search.mode
        else {
            return false;
        };
        match key.code {
            KeyCode::Char(c) if plain => input.push(c),
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Enter => self.replace_prompt_enter(),
            KeyCode::Esc => self.search.mode = None,
            KeyCode::Char('g') if control => self.search.mode = None,
            // Other keys do nothing while typing.
            _ => {}
        }
        true
    }

    fn replace_prompt_enter(&mut self) {
        match self.search.mode.take() {
            // Nothing to replace: give up quietly, as Emacs does.
            Some(Mode::ReplaceFrom(from)) if from.is_empty() => {}
            Some(Mode::ReplaceFrom(from)) => {
                self.search.mode = Some(Mode::ReplaceTo(from, String::new()));
            }
            Some(Mode::ReplaceTo(from, to)) => {
                let origin = self.buffer.editor.get_cursor();
                let state = self.next_match(&from, origin).map(|current| Replacing {
                    from,
                    to,
                    current,
                    count: 0,
                    origin,
                    open: false,
                });
                match state {
                    Some(state) => self.ask_replace(state),
                    None => self.status_bar_text = "Replaced 0 occurrences".into(),
                }
            }
            other => self.search.mode = other,
        }
    }

    // Answers to "replace this match?".
    pub(super) fn replacing_key(&mut self, key: KeyEvent) -> bool {
        let Some(Mode::Replacing(mut state)) = self.search.mode.take() else {
            return false;
        };
        let control = key.modifiers == KeyModifiers::CONTROL;
        let plain = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        match key.code {
            KeyCode::Char('y' | ' ') if plain => {
                let end = self.replace_one(&mut state);
                self.replace_next(state, end);
            }
            KeyCode::Char('n') if plain => {
                let end = state.current.end;
                self.replace_next(state, end);
            }
            KeyCode::Backspace => {
                let end = state.current.end;
                self.replace_next(state, end);
            }
            KeyCode::Char('!') if plain => {
                // Replace this match and every one after it.
                let mut end = self.replace_one(&mut state);
                while let Some(found) = self.next_match(&state.from, end) {
                    state.current = found;
                    end = self.replace_one(&mut state);
                }
                self.finish_replace(state);
            }
            KeyCode::Char('.') if plain => {
                self.replace_one(&mut state);
                self.finish_replace(state);
            }
            KeyCode::Char('q') if plain => self.finish_replace(state),
            KeyCode::Enter | KeyCode::Esc => self.finish_replace(state),
            KeyCode::Char('g') if control => self.finish_replace(state),
            // Any other key ends replacing and is then handled as usual.
            _ => {
                self.finish_replace(state);
                return false;
            }
        }
        true
    }

    // Replace the current match; returns where its replacement ends.
    fn replace_one(&mut self, state: &mut Replacing) -> usize {
        let code = self.buffer.editor.code_mut();
        if !state.open {
            code.tx();
            code.set_state_before(state.origin, None);
            state.open = true;
        }
        let start = state.current.start;
        code.remove(start, state.current.end);
        code.insert(start, &state.to);
        state.count += 1;
        let end = start + state.to.chars().count();
        self.buffer.editor.reset_highlight_cache();
        self.move_cursor(end);
        end
    }

    fn next_match(&self, from: &str, after: usize) -> Option<Range<usize>> {
        let text = self.buffer.editor.get_content();
        find_all(&text, from).into_iter().find(|m| m.start >= after)
    }

    fn replace_next(&mut self, mut state: Replacing, after: usize) {
        match self.next_match(&state.from, after) {
            Some(found) => {
                state.current = found;
                self.ask_replace(state);
            }
            None => self.finish_replace(state),
        }
    }

    fn ask_replace(&mut self, state: Replacing) {
        self.move_cursor(state.current.end);
        self.highlight_matches(&state.from, Some(&state.current));
        self.search.mode = Some(Mode::Replacing(state));
    }

    fn finish_replace(&mut self, state: Replacing) {
        if state.open {
            let cursor = self.buffer.editor.get_cursor();
            let code = self.buffer.editor.code_mut();
            code.set_state_after(cursor, None);
            code.commit();
        }
        self.buffer.editor.remove_marks();
        let plural = if state.count == 1 { "" } else { "s" };
        self.status_bar_text = format!("Replaced {} occurrence{plural}", state.count);
    }
}
