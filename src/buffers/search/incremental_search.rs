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

// Incremental search.

use super::{Mode, find_all};
use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::ops::Range;

pub(super) struct IncrementalSearch {
    query: String,
    forward: bool,

    // Where the search started.
    origin: usize,
    // The match shown now.
    current: Option<Range<usize>>,

    // Nothing more was found while failing.
    failing: bool,
    wrapped: bool,
}

impl IncrementalSearch {
    pub(super) fn prompt(&self) -> String {
        let failing = if self.failing { "Failing " } else { "" };
        let wrapped = if self.wrapped { "Wrapped " } else { "" };
        let backward = if self.forward { "" } else { " backward" };
        format!("{failing}{wrapped}I-search{backward}: {}", self.query)
    }

    // The match to show.
    fn choose(&self, matches: &[Range<usize>], next: bool) -> Option<Range<usize>> {
        let current = self.current.as_ref();
        let found = if self.forward {
            let from = current.map_or(self.origin, |c| c.start + usize::from(next));
            matches.iter().find(|m| m.start >= from)
        } else {
            matches.iter().rev().find(|m| match current {
                Some(c) if next => m.start < c.start,
                Some(c) => m.start <= c.start,
                None => m.end <= self.origin,
            })
        };

        found.cloned()
    }
}

impl App {
    pub(super) fn start_incremental_search(&mut self, forward: bool) {
        let origin = self.buffer.editor.get_cursor();
        self.search.mode = Some(Mode::IncrementalSearch(IncrementalSearch {
            query: String::new(),
            forward,
            origin,
            current: None,
            failing: false,
            wrapped: false,
        }));
    }

    pub(super) fn incremental_search_key(&mut self, key: KeyEvent) -> bool {
        let Some(Mode::IncrementalSearch(mut state)) = self.search.mode.take() else {
            return false;
        };
        let control = key.modifiers == KeyModifiers::CONTROL;
        let plain = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        match key.code {
            KeyCode::Char('s') if control => self.incremental_search_next(&mut state, true),
            KeyCode::Char('r') if control => self.incremental_search_next(&mut state, false),
            KeyCode::Char('g') if control => {
                self.move_cursor(state.origin);
                self.end_incremental_search(state);
                return true;
            }

            KeyCode::Enter => {
                self.end_incremental_search(state);
                return true;
            }

            KeyCode::Backspace => {
                state.query.pop();
                self.incremental_search_update(&mut state, false);
            }

            KeyCode::Char(c) if plain => {
                state.query.push(c);
                self.incremental_search_update(&mut state, false);
            }

            // Unexpected line here :(
            _ => {
                self.end_incremental_search(state);
                return false;
            }
        }
        self.search.mode = Some(Mode::IncrementalSearch(state));
        true
    }

    // C-s or C-r: go on in that direction.
    fn incremental_search_next(&mut self, state: &mut IncrementalSearch, forward: bool) {
        state.forward = forward;
        if state.query.is_empty() {
            state.query = self.search.last.clone();
            self.incremental_search_update(state, false);
        } else {
            self.incremental_search_update(state, true);
        }
    }

    fn incremental_search_update(&mut self, state: &mut IncrementalSearch, next: bool) {
        let matches = find_all(&self.buffer.editor.get_content(), &state.query);
        let mut found = state.choose(&matches, next);
        // After a failing search, one more step starts again from the other end.
        if found.is_none() && next && state.failing {
            let first = if state.forward {
                matches.first()
            } else {
                matches.last()
            };
            found = first.cloned();
            state.wrapped |= found.is_some();
        }
        state.failing = found.is_none() && !state.query.is_empty();
        if let Some(found) = found {
            // Like Emacs: after the match going forward, before it going backward.
            self.move_cursor(if state.forward {
                found.end
            } else {
                found.start
            });
            state.current = Some(found);
        } else if state.query.is_empty() {
            self.move_cursor(state.origin);
            state.current = None;
        }
        self.highlight_matches(&state.query, state.current.as_ref());
    }

    fn end_incremental_search(&mut self, state: IncrementalSearch) {
        if !state.query.is_empty() {
            self.search.last = state.query;
        }
        self.buffer.editor.remove_marks();
    }

    // Put the cursor at offset and scroll it into view.
    pub(super) fn move_cursor(&mut self, offset: usize) {
        self.buffer.editor.set_cursor(offset);
        self.buffer.editor.focus(&self.editor_area);
    }
}
