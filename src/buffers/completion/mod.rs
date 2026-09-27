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

// Completion popup: candidates come from the language server when one is running, otherwise
// from words in the buffer.

mod keys;
mod popup;
mod words;

use crate::{
    app::App,
    lsp::{self, Candidate, Completions},
};
use std::path::PathBuf;

// Completion popup state and the language server feeding it.
#[derive(Default)]
pub(crate) struct Completion {
    // Language server plugins, at most one per language. Empty until `set_plugins`.
    plugins: Vec<lsp::Plugin>,
    client: Option<lsp::Client>,
    // Language and path the server was chosen for, so a failed start is not retried every frame.
    attached: Option<(String, PathBuf)>,
    candidates: Vec<Candidate>,
    visible: Vec<usize>,
    selected: usize,
    active: bool,
    // Opened with M-/ rather than by typing: shows "Loading…" and reports empty results.
    manual: bool,
    // Popping up while typing was switched off with C-c M-l.
    auto_off: bool,
    // Cursor at the time of an unanswered server request.
    waiting: Option<usize>,
    incomplete: bool,
    word_start: usize,
    // Cursor when the shown candidates were computed; their ranges are relative to it.
    requested_at: usize,
}

impl Completion {
    // True while a server answer is expected, so the UI loop should poll more often.
    pub(crate) fn is_waiting(&self) -> bool {
        self.active && self.waiting.is_some()
    }

    pub(crate) fn auto_enabled(&self) -> bool {
        !self.auto_off
    }

    pub(crate) fn set_plugins(&mut self, plugins: Vec<lsp::Plugin>) {
        self.plugins = plugins;
        self.attached = None;
    }

    fn open(&mut self, manual: bool) {
        self.close();
        self.active = true;
        self.manual = manual;
    }

    fn close(&mut self) {
        self.active = false;
        self.waiting = None;
        self.candidates.clear();
        self.visible.clear();
    }

    fn step(&mut self, forward: bool) {
        let count = self.visible.len().max(1);
        self.selected = if forward {
            (self.selected + 1) % count
        } else {
            (self.selected + count - 1) % count
        };
    }

    // Candidates that would change nothing (the word is already typed out) are hidden.
    fn refilter(&mut self, prefix: &str) {
        self.visible = (0..self.candidates.len())
            .filter(|&index| {
                let candidate = &self.candidates[index];
                candidate.matches(prefix) && candidate.text != prefix
            })
            .collect();
        self.selected = 0;
    }
}

impl App {
    // Keep the language server matched to the buffer, forward edits and collect answers.
    pub(crate) fn sync_lsp(&mut self) {
        let language = self.buffer.editor.code_ref().lang().to_owned();
        let target = self.buffer.path.clone().map(|path| (language, path));
        if self.completion.attached != target {
            self.attach_lsp(target);
        }

        let Some(client) = &mut self.completion.client else {
            return;
        };
        client.change(&self.buffer.editor.get_content());
        let answer = client.poll();
        let alive = client.is_alive();
        if let Some(answer) = answer {
            self.receive(answer);
        }
        if !alive {
            self.completion.client = None;
            if self.completion.is_waiting() {
                self.request_completion(None);
            }
        }
    }

    fn attach_lsp(&mut self, target: Option<(String, PathBuf)>) {
        let completion = &mut self.completion;
        completion.close();
        completion.attached = target.clone();
        let Some((language, path)) = target else {
            completion.client = None;
            return;
        };
        // A running server is reused when the new language's plugin runs the same command,
        // as C and C++ both do with clangd.
        let plugin = completion
            .plugins
            .iter()
            .find(|plugin| plugin.serves(&language));
        let running = completion.client.as_ref().map(lsp::Client::command);
        if running != plugin.map(|plugin| plugin.command.as_slice()) {
            completion.client = plugin.and_then(|plugin| lsp::Client::start(plugin, &path).ok());
        }
        let language_id = lsp::language_id(&language).unwrap_or("plaintext");
        let text = self.buffer.editor.get_content();
        if let Some(client) = &mut completion.client
            && client.open(&path, language_id, &text).is_err()
        {
            completion.client = None;
        }
    }

    // Compute candidates for the word before the cursor: ask the server when one is ready,
    // otherwise offer words from the buffer. `trigger` is the trigger character just typed.
    fn request_completion(&mut self, trigger: Option<char>) {
        let text = self.buffer.editor.get_content();
        let cursor = self.buffer.editor.get_cursor();
        let start = words::word_start(&text, cursor);
        self.completion.word_start = start;
        if let Some(client) = &mut self.completion.client {
            client.change(&text);
            if client.complete(start..cursor, trigger) {
                self.completion.waiting = Some(cursor);
                return;
            }
        }
        self.receive(Completions {
            candidates: words::buffer_words(&text, start..cursor),
            incomplete: false,
        });
        self.completion.requested_at = cursor;
    }

    fn receive(&mut self, answer: Completions) {
        if !self.completion.active {
            return;
        }
        if let Some(cursor) = self.completion.waiting.take() {
            self.completion.requested_at = cursor;
        }
        self.completion.candidates = answer.candidates;
        self.completion.incomplete = answer.incomplete;
        self.completion.refilter(&self.completion_prefix());
        if self.completion.visible.is_empty() {
            if self.completion.manual {
                self.status_bar_text = "No completions".into();
            }
            self.completion.close();
        }
    }

    fn completion_prefix(&self) -> String {
        let cursor = self.buffer.editor.get_cursor();
        let start = self.completion.word_start.min(cursor);
        self.buffer.editor.get_content_slice(start, cursor)
    }

    // Replace the candidate's range with its text as one undoable edit.
    fn accept_completion(&mut self) -> bool {
        let completion = &self.completion;
        let Some(candidate) = completion
            .visible
            .get(completion.selected)
            .map(|&index| completion.candidates[index].clone())
        else {
            self.completion.close();
            return false;
        };

        // Characters typed since the request extend (or shrink) the replaced range.
        let editor = &mut self.buffer.editor;
        let cursor = editor.get_cursor();
        let length = editor.code_ref().len_chars();
        let start = candidate.range.start.min(length);
        let end = (candidate.range.end + cursor)
            .saturating_sub(completion.requested_at)
            .clamp(start, length);
        let code = editor.code_mut();
        code.tx();
        code.set_state_before(cursor, None);
        code.remove(start, end);
        code.insert(start, &candidate.text);
        let cursor = start + candidate.text.chars().count();
        code.set_state_after(cursor, None);
        code.commit();
        editor.set_cursor(cursor);
        editor.set_selection(None);
        editor.reset_highlight_cache();
        editor.focus(&self.editor_area);
        self.completion.close();
        true
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::layout::Rect;

    pub(crate) fn press(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
        app.handle_key(KeyEvent::new(code, modifiers));
    }

    pub(crate) fn type_text(app: &mut App, text: &str) {
        app.editor_area = Rect::new(1, 1, 78, 22);
        for c in text.chars() {
            let code = if c == '\n' {
                KeyCode::Enter
            } else {
                KeyCode::Char(c)
            };
            press(app, code, KeyModifiers::NONE);
        }
    }

    pub(crate) fn labels(app: &App) -> Vec<&str> {
        let completion = &app.completion;
        let visible = completion.visible.iter();
        visible
            .map(|&index| completion.candidates[index].label.as_str())
            .collect()
    }

    #[test]
    fn late_server_answers_cover_text_typed_while_waiting() {
        let mut app = App::default();
        app.completion.auto_off = true;
        type_text(&mut app, "pri");
        app.completion.open(true);
        app.completion.waiting = Some(3);
        type_text(&mut app, "n");
        app.receive(Completions {
            candidates: vec![
                Candidate::plain("printf", 0..3),
                Candidate::plain("prin", 0..3),
            ],
            incomplete: false,
        });
        assert_eq!(labels(&app), ["printf"], "typed-out word is hidden");
        press(&mut app, KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(app.buffer.editor.get_content(), "printf");
        press(&mut app, KeyCode::Char('z'), KeyModifiers::CONTROL);
        assert_eq!(app.buffer.editor.get_content(), "prin");
    }

    #[test]
    fn empty_answers_are_reported_only_when_asked_for() {
        let mut app = App::default();
        for manual in [false, true] {
            app.completion.open(manual);
            app.completion.waiting = Some(0);
            app.receive(Completions::default());
            assert!(!app.completion.active);
            assert_eq!(app.status_bar_text == "No completions", manual);
        }
    }
}
