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

use crate::buffers::editor::Buffer;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use std::{io, path::PathBuf};

// The state struct of editor
#[derive(Default)]
pub(crate) struct App {
    pub(crate) buffer: Buffer,
    exit_flag: bool,
    ctrl_x_wait_flag: bool,
    pub(crate) save_path_input: Option<String>,
    pub(crate) status_bar_text: String,
}

// Implementation of the app struct
impl App {
    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // The main loop: While the exit flag is false, keep ratatui running
        while !self.exit_flag {
            // New frame
            terminal.draw(|frame| self.draw(frame))?;

            // Wait for event; ignore key release to avoid handling a keystroke twice.
            if let Event::Key(key) = event::read()?
                && key.kind != KeyEventKind::Release
            {
                // Update status_bar_text
                self.handle_key(key);
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // While entering a filename, keystrokes must not change the buffer.
        if let Some(input) = &mut self.save_path_input {
            match key.code {
                KeyCode::Esc => {
                    self.save_path_input = None;
                    self.status_bar_text = "Save cancelled".into();
                }
                KeyCode::Enter => {
                    if input.trim().is_empty() {
                        self.status_bar_text = "Enter a file path".into();
                        return;
                    }

                    let path = PathBuf::from(input.as_str());
                    match self.buffer.save_as(path) {
                        Ok(()) => {
                            self.save_path_input = None;
                            self.status_bar_text = "Saved".into();
                        }
                        Err(err) => self.status_bar_text = format!("Save failed: {err}"),
                    }
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(c)
                    if !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    input.push(c);
                }
                _ => {}
            }
            return;
        }

        // C-x is responsible for C-x C-c (exit) and C-x C-s (save)
        if self.ctrl_x_wait_flag {
            self.ctrl_x_wait_flag = false;
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    // Hit "save"
                    KeyCode::Char('s') => {
                        // If we've got file path the write the file.
                        if self.buffer.path.is_some() {
                            match self.buffer.save() {
                                Ok(()) => self.status_bar_text = "Saved".into(),
                                Err(err) => self.status_bar_text = format!("Save failed: {err}"),
                            }
                        } else {
                            // Else, manually ask for user input.
                            self.save_path_input = Some(String::new());
                            self.status_bar_text =
                                "Enter a file path => Enter to save, Esc to cancel".into();
                        }
                        return;
                    }

                    // Hit "exit"
                    KeyCode::Char('c') => {
                        self.exit_flag = true;
                        return;
                    }
                    _ => {}
                }
            }
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            if key.code == KeyCode::Char('x') {
                self.ctrl_x_wait_flag = true;
                self.status_bar_text = "C-x pressed".into();
            }
            return;
        }

        match key.code {
            // Directly write normal character into buffer unless you're pressing ALT key.
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::ALT) => {
                self.buffer.text.push(c);
            }
            // Pressing ENTER pushes a \n to buffer.
            KeyCode::Enter => self.buffer.text.push('\n'),
            // Pressing BACKSPACE results a pop of last char in buffer.
            KeyCode::Backspace => {
                self.buffer.text.pop();
            }
            // Not handling anything other than those.
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_requires_ctrl_x_then_ctrl_c() {
        let mut app = App::default();
        app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(!app.exit_flag);

        app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE));
        assert!(!app.exit_flag);

        app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.exit_flag);
    }

    #[test]
    fn unnamed_buffer_prompts_then_remembers_path() {
        let mut app = App::default();
        app.buffer.text = "hello\n".into();
        app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert!(app.save_path_input.is_some());

        let path = std::env::temp_dir().join(format!("ptre-save-test-{}", std::process::id()));
        app.save_path_input = Some(path.to_string_lossy().into_owned());
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert_eq!(app.buffer.path.as_ref(), Some(&path));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello\n");

        app.buffer.text.push_str("world");
        app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert!(app.save_path_input.is_none());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello\nworld");
        std::fs::remove_file(path).unwrap();
    }
}
