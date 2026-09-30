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

//! Emacs-style search and replace.

mod incremental_search;
mod replace;
#[cfg(test)]
mod tests;

use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use incremental_search::IncrementalSearch;
use replace::Replacing;
use std::ops::Range;
const CURRENT_MATCH: &str = "#4f6f9f";
const OTHER_MATCH: &str = "#2c3a4c";

#[derive(Default)]
pub(crate) struct Search {
    mode: Option<Mode>,
    // The last search string, reused by C-s or C-r on an empty search.
    last: String,
}

enum Mode {
    IncrementalSearch(IncrementalSearch),
    ReplaceFrom(String),
    ReplaceTo(String, String),
    Replacing(Replacing),
}

impl Search {
    // The status bar text.
    pub(crate) fn prompt(&self) -> Option<String> {
        Some(match self.mode.as_ref()? {
            Mode::IncrementalSearch(incremental_search) => incremental_search.prompt(),
            Mode::ReplaceFrom(from) => format!("Query replace: {from}█"),
            Mode::ReplaceTo(from, to) => format!("Query replace {from} with: {to}█"),
            Mode::Replacing(replacing) => replacing.prompt(),
        })
    }

    // True while the user types into the status bar.
    // This hides the cursor.
    pub(crate) fn reads_input(&self) -> bool {
        matches!(self.mode, Some(Mode::ReplaceFrom(_) | Mode::ReplaceTo(..)))
    }
}

impl App {
    // Keys of search and replace.
    pub(crate) fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        // C-c chords belong to completion and the checks; go to line keeps its own keys.
        if self.ctrl_c_wait_flag || self.goto_line.is_busy() {
            return false;
        }
        match self.search.mode {
            None => self.start_search(key),
            Some(Mode::IncrementalSearch(_)) => self.incremental_search_key(key),
            Some(Mode::ReplaceFrom(_) | Mode::ReplaceTo(..)) => self.replace_prompt_key(key),
            Some(Mode::Replacing(_)) => self.replacing_key(key),
        }
    }

    fn start_search(&mut self, key: KeyEvent) -> bool {
        let control = key.modifiers == KeyModifiers::CONTROL;
        // M-% is Alt + Shift + 5.
        let meta = key.modifiers.contains(KeyModifiers::ALT)
            && !key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('s') if control => self.start_incremental_search(true),
            KeyCode::Char('r') if control => self.start_incremental_search(false),
            KeyCode::Char('%') if meta => self.start_replace(),
            _ => return false,
        }
        self.completion.close();
        true
    }

    // Mark every match of query, the current one brighter than the others.
    fn highlight_matches(&mut self, query: &str, current: Option<&Range<usize>>) {
        let text = self.buffer.editor.get_content();
        let marks: Vec<_> = find_all(&text, query)
            .into_iter()
            .map(|found| {
                let color = if Some(&found) == current {
                    CURRENT_MATCH
                } else {
                    OTHER_MATCH
                };
                (found.start, found.end, color)
            })
            .collect();
        if marks.is_empty() {
            self.buffer.editor.remove_marks();
        } else {
            self.buffer.editor.set_marks(marks);
        }
    }
}

// Char ranges of the matches of query in text.
fn find_all(text: &str, query: &str) -> Vec<Range<usize>> {
    let fold = !query.chars().any(char::is_uppercase);
    let same = |t: char, q: char| t == q || (fold && t.to_lowercase().eq(q.to_lowercase()));
    let text: Vec<char> = text.chars().collect();
    let query: Vec<char> = query.chars().collect();
    let mut found = Vec::new();
    let mut start = 0;
    while !query.is_empty() && start + query.len() <= text.len() {
        if query.iter().zip(&text[start..]).all(|(&q, &t)| same(t, q)) {
            found.push(start..start + query.len());
            start += query.len();
        } else {
            start += 1;
        }
    }
    found
}
