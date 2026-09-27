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
use ratatui::{DefaultTerminal, layout::Rect};
use ratatui_code_editor::actions::{Delete, InsertText, Redo};
use std::{
    io,
    path::PathBuf,
    time::{Duration, Instant},
};

// The state struct of editor
#[derive(Default)]
pub(crate) struct App {
    pub(crate) buffer: Buffer,
    pub(crate) editor_area: Rect,
    exit_flag: bool,
    ctrl_x_wait_flag: bool,
    kill_ring: Option<String>,
    pub(crate) overwrite_confirm: bool,
    pub(crate) save_path_input: Option<String>,
    pub(crate) status_bar_text: String,
}

// Implementation of the app struct
impl App {
    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // Watch the file each 500ms.
        let interval = Duration::from_millis(500);
        let mut last_disk_check = Instant::now() - interval;

        // The main loop: While the exit flag is false, keep ratatui running
        while !self.exit_flag {
            // Timer for check file state.
            if last_disk_check.elapsed() >= interval {
                self.buffer.check_disk();
                last_disk_check = Instant::now();
            }

            // New frame
            terminal.draw(|frame| self.draw(frame))?;

            // Wait for event; ignore key release to avoid handling a keystroke twice.
            if event::poll(interval.saturating_sub(last_disk_check.elapsed()))?
                && let Event::Key(key) = event::read()?
                && key.kind != KeyEventKind::Release
            {
                // Update status_bar_text
                self.handle_key(key);
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Conformation modal "dialog" for overwriting document.
        if self.overwrite_confirm {
            match key.code {
                // Press Y or ENTER for conforming.
                KeyCode::Char('y' | 'Y') | KeyCode::Enter
                    if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
                {
                    self.overwrite_confirm = false;
                    match self.buffer.save() {
                        Ok(()) => self.status_bar_text = "Saved".into(),
                        Err(err) => self.status_bar_text = format!("Save failed: {err}"),
                    }
                }

                // Press N or ESC to cancel.
                KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                    self.overwrite_confirm = false;
                    self.status_bar_text = "Save cancelled".into();
                }

                // Ignoring all other inputs.
                _ => {}
            }
            return;
        }

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
                            self.buffer.check_disk();
                            if self.buffer.obsolete {
                                self.overwrite_confirm = true;
                                self.status_bar_text =
                                    "File changed on disk => confirm overwrite".into();
                            } else {
                                match self.buffer.save() {
                                    Ok(()) => self.status_bar_text = "Saved".into(),
                                    Err(err) => {
                                        self.status_bar_text = format!("Save failed: {err}")
                                    }
                                }
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

        // Reserve C-x for application commands rather than the editor's cut binding.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('x') {
            self.ctrl_x_wait_flag = true;
            self.status_bar_text = "C-x pressed".into();
            return;
        }

        // Emacs-style pasteboard shortcuts.
        match (key.code, key.modifiers) {
            // M-w => Copy from kill ring.
            (KeyCode::Char('w'), KeyModifiers::ALT | KeyModifiers::CONTROL) => {
                if let Some(text) = self.buffer.editor.get_selection_text() {
                    let _ = self.buffer.editor.set_clipboard(&text);
                    self.kill_ring = Some(text);
                    if key.modifiers == KeyModifiers::CONTROL {
                        self.buffer.editor.apply(Delete);
                    }
                }
            }

            // C-y => "yanking" from kill ring.
            (KeyCode::Char('y'), KeyModifiers::CONTROL) => {
                if let Some(text) = &self.kill_ring {
                    self.buffer.editor.apply(InsertText { text: text.clone() });
                }
            }
            (KeyCode::Char('c' | 'v'), KeyModifiers::CONTROL) => return,
            (KeyCode::Char('Z' | 'z'), modifiers)
                if modifiers == KeyModifiers::CONTROL | KeyModifiers::SHIFT =>
            {
                self.buffer.editor.apply(Redo)
            }
            // Shouldn't be reached: Unhandled keys
            _ => {
                if let Err(err) = self.buffer.editor.input(key, &self.editor_area) {
                    self.status_bar_text = format!("Editor error: {err}");
                }
                return;
            }
        }
        self.buffer.editor.focus(&self.editor_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestFile(PathBuf);
    impl TestFile {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "ptre-confirm-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::write(&path, "original").unwrap();
            Self(path)
        }
    }
    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn save_keys(app: &mut App) {
        app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    }

    fn type_text(app: &mut App, text: &str) {
        app.editor_area = Rect::new(1, 1, 78, 22);
        for c in text.chars() {
            let code = if c == '\n' {
                KeyCode::Enter
            } else {
                KeyCode::Char(c)
            };
            app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
        }
    }

    #[test]
    fn cursor_editing_and_undo_use_editor_buffer() {
        let mut app = App::default();
        type_text(&mut app, "ac");
        app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        type_text(&mut app, "b");
        assert_eq!(app.buffer.editor.get_content(), "abc");
        app.handle_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        assert_eq!(app.buffer.editor.get_content(), "ac");
    }

    #[test]
    fn emacs_region_shortcuts_replace_default_clipboard_bindings() {
        let mut app = App::default();
        type_text(&mut app, "abc");
        app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));

        app.handle_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::ALT));
        assert_eq!(app.kill_ring.as_deref(), Some("bc"));
        assert_eq!(app.buffer.editor.get_content(), "abc");

        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL));
        assert_eq!(app.buffer.editor.get_content(), "abc");

        app.buffer.editor.clear_selection();
        app.buffer.editor.set_cursor(3);
        app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL));
        assert_eq!(app.buffer.editor.get_content(), "abcbc");

        app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        app.handle_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL));
        assert_eq!(app.buffer.editor.get_content(), "abc");
        app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL));
        assert_eq!(app.buffer.editor.get_content(), "abcbc");
    }

    #[test]
    fn save_prompt_does_not_edit_content() {
        let mut app = App::default();
        type_text(&mut app, "hello");
        app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
        type_text(&mut app, "cancelled.rs");
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(app.save_path_input.is_none());
        assert_eq!(app.buffer.editor.get_content(), "hello");
    }

    #[test]
    fn obsolete_file_requires_confirmation_and_cancel_keeps_disk_intact() {
        let file = TestFile::new();
        let mut app = App::default();
        app.buffer.open(file.0.clone()).unwrap();
        app.buffer.editor.set_content("my edits");
        std::fs::write(&file.0, "external edits").unwrap();

        // No timer tick is needed: saving checks the disk immediately.
        save_keys(&mut app);
        assert!(app.overwrite_confirm);
        assert_eq!(std::fs::read_to_string(&file.0).unwrap(), "external edits");
        app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
        assert_eq!(app.buffer.editor.get_content(), "my edits");
        assert!(app.overwrite_confirm);

        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!app.overwrite_confirm);
        assert!(app.buffer.is_dirty());
        assert!(app.buffer.obsolete);
        assert_eq!(std::fs::read_to_string(&file.0).unwrap(), "external edits");
    }

    #[test]
    fn confirming_overwrites_obsolete_file_and_resets_indicators() {
        let file = TestFile::new();
        let mut app = App::default();
        app.buffer.open(file.0.clone()).unwrap();
        app.buffer.editor.set_content("my edits");
        std::fs::write(&file.0, "external edits").unwrap();
        save_keys(&mut app);
        assert!(app.overwrite_confirm);
        app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));
        assert!(!app.overwrite_confirm);
        assert!(!app.buffer.is_dirty());
        assert!(!app.buffer.obsolete);
        assert_eq!(std::fs::read_to_string(&file.0).unwrap(), "my edits");
    }

    #[test]
    fn enter_confirms_overwrite() {
        let file = TestFile::new();
        let mut app = App::default();
        app.buffer.open(file.0.clone()).unwrap();
        app.buffer.editor.set_content("my edits");
        std::fs::write(&file.0, "external edits").unwrap();
        save_keys(&mut app);

        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(!app.overwrite_confirm);
        assert_eq!(std::fs::read_to_string(&file.0).unwrap(), "my edits");
    }

    #[test]
    fn confirmation_dialog_is_rendered_over_editor() {
        use ratatui::{Terminal, backend::TestBackend};
        let mut app = App {
            overwrite_confirm: true,
            ..App::default()
        };
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let screen = terminal.backend().buffer();
        let text: String = (0..24)
            .flat_map(|y| (0..80).map(move |x| screen[(x, y)].symbol().to_owned()))
            .collect();
        assert!(text.contains("Confirm overwrite"));
        assert!(text.contains("File has been changed on disk. Overwrite it?"));
        assert!(text.contains("Y/ENTER"));
    }

    #[test]
    fn renders_editor_and_updated_shortcuts() {
        use ratatui::{Terminal, backend::TestBackend};
        let mut app = App::default();
        type_text(&mut app, "fn main() {}");
        app.buffer.obsolete = true;
        let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert_eq!(app.editor_area, Rect::new(1, 1, 98, 21));
        let screen = terminal.backend().buffer();
        let title: String = (0..100).map(|x| screen[(x, 0)].symbol()).collect();
        assert!(title.contains("· DIRTY BUFFER"));
        assert!(title.contains("· OBSOLETE FILE"));
        let status: String = (0..100).map(|x| screen[(x, 23)].symbol()).collect();
        assert!(status.contains("C-x C-c: quit"));
        assert!(
            app.buffer
                .editor
                .get_visible_cursor(&app.editor_area)
                .is_some()
        );
    }

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
        type_text(&mut app, "hello\n");
        app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert!(app.save_path_input.is_some());

        let path = std::env::temp_dir().join(format!("ptre-save-test-{}", std::process::id()));
        app.save_path_input = Some(path.to_string_lossy().into_owned());
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert_eq!(app.buffer.path.as_ref(), Some(&path));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello\n");

        type_text(&mut app, "world");
        app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
        app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert!(app.save_path_input.is_none());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello\nworld");
        std::fs::remove_file(path).unwrap();
    }
}
