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
fn emacs_undo_and_redo_shortcuts() {
    let mut app = App::default();
    type_text(&mut app, "a");
    app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
    app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE));
    assert_eq!(app.buffer.editor.get_content(), "");
    app.handle_key(KeyEvent::new(
        KeyCode::Char('_'),
        KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT,
    ));
    assert_eq!(app.buffer.editor.get_content(), "a");
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
    // With one tab, the tab bar is hidden and the editor starts right below the border.
    assert_eq!(app.editor_area, Rect::new(1, 3, 98, 16));
    let screen = terminal.backend().buffer();
    let row: String = (0..100).map(|x| screen[(x, 3)].symbol()).collect();
    assert!(!row.contains('❃'), "{row}");
    let title: String = (0..100).map(|x| screen[(x, 2)].symbol()).collect();
    assert!(title.starts_with("╭─ [No Name] · DIRTY BUFFER"));
    assert!(title.contains("· DIRTY BUFFER"));
    assert!(title.contains("· OBSOLETE FILE"));
    assert_eq!(screen[(0, 2)].symbol(), "╭");
    assert_eq!(screen[(99, 2)].symbol(), "╮");
    assert_eq!(screen[(0, 21)].symbol(), "╰");
    assert_eq!(screen[(99, 21)].symbol(), "╯");
    assert_eq!(screen[(0, 2)].fg, ratatui::style::Color::White);
    assert_eq!(screen[(0, 10)].fg, ratatui::style::Color::White);
    assert_eq!(screen[(99, 10)].fg, ratatui::style::Color::White);
    let status: String = (0..100).map(|x| screen[(x, 23)].symbol()).collect();
    assert!(status.contains("F10: menu"));
    assert!(!status.contains("Buffer command"));
    for x in 0..100 {
        assert_eq!(screen[(x, 22)].symbol(), "─");
        assert_eq!(screen[(x, 21)].fg, ratatui::style::Color::White);
        assert_eq!(screen[(x, 22)].fg, ratatui::style::Color::Green);
        assert_eq!(screen[(x, 22)].bg, ratatui::style::Color::Reset);
        assert_eq!(screen[(x, 23)].bg, ratatui::style::Color::Reset);
        assert_eq!(screen[(x, 23)].fg, ratatui::style::Color::Green);
    }
    assert!(
        app.buffer
            .editor
            .get_visible_cursor(&app.editor_area)
            .is_some()
    );

    // A second tab brings the tab bar back.
    app.open_tab("Cargo.toml".into()).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    assert_eq!(app.editor_area, Rect::new(1, 4, 98, 15));
    let screen = terminal.backend().buffer();
    let tabs: String = (0..100).map(|x| screen[(x, 3)].symbol()).collect();
    assert!(tabs.starts_with("│   [No Name] ●  ❃ Cargo.toml"), "{tabs}");
}

#[test]
fn echoes_emacs_shortcuts_after_menu_hint() {
    use ratatui::{Terminal, backend::TestBackend};
    let mut app = App::default();
    let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
    let status = |app: &mut App, terminal: &mut Terminal<TestBackend>| {
        terminal.draw(|frame| app.draw(frame)).unwrap();
        (0..100)
            .map(|x| terminal.backend().buffer()[(x, 23)].symbol().to_owned())
            .collect::<String>()
    };
    assert!(status(&mut app, &mut terminal).contains("F10: menu"));
    assert!(!status(&mut app, &mut terminal).contains("Buffer command:"));
    app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
    assert!(status(&mut app, &mut terminal).contains("Buffer command: C-x"));
    app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE));
    assert!(status(&mut app, &mut terminal).contains("Buffer command: C-x u"));
    app.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
    assert!(!status(&mut app, &mut terminal).contains("Buffer command: C-x"));
    app.handle_key(KeyEvent::new(
        KeyCode::Char('_'),
        KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT,
    ));
    assert!(status(&mut app, &mut terminal).contains("C-M-_"));
    app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
    assert!(status(&mut app, &mut terminal).contains("F10"));
}

#[test]
fn editor_title_shows_only_file_name_and_indicators() {
    use ratatui::{Terminal, backend::TestBackend};
    let mut app = App::default();
    app.buffer.open(PathBuf::from("Cargo.toml")).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let screen = terminal.backend().buffer();
    let title: String = (0..80).map(|x| screen[(x, 2)].symbol()).collect();
    assert!(title.starts_with("╭─ Cargo.toml ─"));
    assert!(!title.contains("TOML"));
    assert!(!title.contains("PTRE UNSTABLE"));
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
