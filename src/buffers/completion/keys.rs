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

use super::words::is_word_char;
use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// Word length at which typing opens the popup by itself.
const AUTO_MIN_CHARS: usize = 2;

impl App {
    // Returns true when the key was consumed by completion.
    pub(crate) fn handle_completion_key(&mut self, key: KeyEvent) -> bool {
        let control = key.modifiers == KeyModifiers::CONTROL;
        let alt = key.modifiers == KeyModifiers::ALT;
        let plain = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);

        // C-c starts a mode-specific chord; the LSP commands are the only ones so far.
        if self.ctrl_c_wait_flag {
            self.ctrl_c_wait_flag = false;
            if key.code == KeyCode::Char('l') && alt {
                self.toggle_auto_completion();
            }
            return true;
        }
        if key.code == KeyCode::Char('c') && control {
            self.completion.close();
            self.ctrl_c_wait_flag = true;
            return true;
        }
        // M-/ opens or closes the popup by hand.
        if key.code == KeyCode::Char('/') && alt {
            if self.completion.active {
                self.completion.close();
            } else {
                self.completion.open(true);
                self.request_completion(None);
            }
            return true;
        }

        if self.completion.active {
            // Until candidates are shown, only typing keeps the popup alive.
            let shown = !self.completion.visible.is_empty();
            match key.code {
                // Typing more of the word narrows the list instead of closing it.
                KeyCode::Char(c) if plain && is_word_char(c) => {
                    self.type_into_completion(key);
                    return true;
                }
                KeyCode::Backspace if plain => {
                    self.type_into_completion(key);
                    return true;
                }
                KeyCode::Up if shown => return self.step_completion(false),
                KeyCode::Char('p') if control && shown => return self.step_completion(false),
                KeyCode::Down if shown => return self.step_completion(true),
                KeyCode::Char('n') if control && shown => return self.step_completion(true),
                KeyCode::Enter | KeyCode::Tab if shown => return self.accept_completion(),
                KeyCode::Esc => {
                    self.completion.close();
                    return true;
                }
                KeyCode::Char('g') if control => {
                    self.completion.close();
                    return true;
                }
                _ => self.completion.close(),
            }
        }

        // With the popup closed, typing may open it.
        match key.code {
            KeyCode::Char(c) if plain && !self.completion.auto_off => {
                self.input_key(key);
                self.auto_complete(c);
                true
            }
            _ => false,
        }
    }

    fn toggle_auto_completion(&mut self) {
        self.completion.auto_off = !self.completion.auto_off;
        self.status_bar_text = if self.completion.auto_off {
            "Auto completion off".into()
        } else {
            "Auto completion on".into()
        };
    }

    fn step_completion(&mut self, forward: bool) -> bool {
        self.completion.step(forward);
        true
    }

    // Open the popup after a server trigger character, or once the word is long enough.
    fn auto_complete(&mut self, typed: char) {
        let trigger = self
            .completion
            .client
            .as_ref()
            .is_some_and(|client| client.is_trigger(typed));
        let cursor = self.buffer.editor.get_cursor();
        let recent = self
            .buffer
            .editor
            .get_content_slice(cursor.saturating_sub(AUTO_MIN_CHARS), cursor);
        let long_enough =
            recent.chars().count() == AUTO_MIN_CHARS && recent.chars().all(is_word_char);
        if trigger || (is_word_char(typed) && long_enough) {
            self.completion.open(false);
            self.request_completion(trigger.then_some(typed));
        }
    }

    fn type_into_completion(&mut self, key: KeyEvent) {
        self.input_key(key);
        if self.buffer.editor.get_cursor() < self.completion.word_start {
            self.completion.close();
        } else if self.completion.incomplete {
            // Keep showing the old list until the new answer arrives.
            self.request_completion(None);
        } else {
            self.completion.refilter(&self.completion_prefix());
            if self.completion.visible.is_empty() && self.completion.waiting.is_none() {
                self.completion.close();
            }
        }
    }

    fn input_key(&mut self, key: KeyEvent) {
        if let Err(err) = self.buffer.editor.input(key, &self.editor_area) {
            self.status_bar_text = format!("Editor error: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{labels, press, type_text};
    use super::*;

    fn complete(app: &mut App) {
        press(app, KeyCode::Char('/'), KeyModifiers::ALT);
    }

    fn manual_app() -> App {
        let mut app = App::default();
        app.completion.auto_off = true;
        app
    }

    #[test]
    fn manual_completion_is_nearest_first_and_toggles() {
        let mut app = manual_app();
        type_text(&mut app, "hello 42 x help\nhe");
        assert!(!app.completion.active);
        complete(&mut app);
        assert_eq!(labels(&app), ["help", "hello"]);
        complete(&mut app);
        assert!(!app.completion.active, "M-/ closes an open popup");
        complete(&mut app);
        press(&mut app, KeyCode::Down, KeyModifiers::NONE);
        press(&mut app, KeyCode::Enter, KeyModifiers::NONE);
        assert!(!app.completion.active);
        assert_eq!(app.buffer.editor.get_content(), "hello 42 x help\nhello");
        assert_eq!(app.buffer.editor.get_cursor(), 21);
    }

    #[test]
    fn typing_narrows_and_other_keys_close() {
        let mut app = manual_app();
        type_text(&mut app, "hello help\nhe");
        complete(&mut app);
        type_text(&mut app, "ll");
        assert_eq!(labels(&app), ["hello"]);
        press(&mut app, KeyCode::Backspace, KeyModifiers::NONE);
        press(&mut app, KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(labels(&app), ["help", "hello"]);
        press(&mut app, KeyCode::Esc, KeyModifiers::NONE);
        assert!(!app.completion.active);

        complete(&mut app);
        press(&mut app, KeyCode::Left, KeyModifiers::NONE);
        assert!(!app.completion.active);
        assert_eq!(
            app.buffer.editor.get_cursor(),
            12,
            "key still reaches the editor"
        );
        type_text(&mut app, "\nzzz");
        complete(&mut app);
        assert!(!app.completion.active);
        assert_eq!(app.status_bar_text, "No completions");
    }

    #[test]
    fn popup_opens_while_typing_and_enter_after_a_full_word_is_a_newline() {
        let mut app = App::default();
        type_text(&mut app, "hello help\nh");
        assert!(!app.completion.active, "one character is not enough");
        type_text(&mut app, "e");
        assert_eq!(labels(&app), ["help", "hello"]);
        type_text(&mut app, "llo");
        assert!(!app.completion.active, "nothing left to complete");
        press(&mut app, KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(app.buffer.editor.get_content(), "hello help\nhello\n");

        type_text(&mut app, "hel");
        assert_eq!(labels(&app), ["hello", "help"]);
        press(&mut app, KeyCode::Down, KeyModifiers::NONE);
        press(&mut app, KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(app.buffer.editor.get_content(), "hello help\nhello\nhelp");
        type_text(&mut app, " x");
        assert!(!app.completion.active);
    }

    #[test]
    fn ctrl_c_meta_l_toggles_auto_popup() {
        let mut app = App::default();
        press(&mut app, KeyCode::Char('c'), KeyModifiers::CONTROL);
        press(&mut app, KeyCode::Char('l'), KeyModifiers::ALT);
        assert_eq!(app.buffer_command, "C-c M-l");
        assert_eq!(app.status_bar_text, "Auto completion off");
        type_text(&mut app, "hello he");
        assert!(!app.completion.active);
        complete(&mut app);
        assert_eq!(labels(&app), ["hello"], "M-/ still works");

        press(&mut app, KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(!app.completion.active);
        press(&mut app, KeyCode::Char('l'), KeyModifiers::ALT);
        assert_eq!(app.status_bar_text, "Auto completion on");
        type_text(&mut app, "\nhe");
        assert_eq!(labels(&app), ["hello"]);

        // Unknown C-c chords do nothing, including C-c C-v.
        press(&mut app, KeyCode::Char('c'), KeyModifiers::CONTROL);
        press(&mut app, KeyCode::Char('v'), KeyModifiers::CONTROL);
        assert_eq!(app.buffer.editor.get_content(), "hello he\nhe");
        assert!(app.completion.auto_enabled());
    }
}
