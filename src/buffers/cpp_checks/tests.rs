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
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ptre-cpp-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
}

fn cpp_app(dir: &Directory) -> App {
    let mut app = App::default();
    let path = dir.0.join("test.cpp");
    std::fs::write(&path, "int main() {}\n").unwrap();
    app.buffer.open(path).unwrap();
    app.cpp_checks.config_path = Some(dir.0.join("settings.json"));
    app
}

#[test]
fn preferences_default_off_and_round_trip() {
    let dir = Directory::new();
    let path = dir.0.join("settings.json");
    let defaults = Settings::load(&path).unwrap();
    assert!(!defaults.tidy && !defaults.cpplint);
    let settings = Settings {
        tidy: true,
        cpplint: true,
        cpplint_path: "/path with spaces/cpplint".into(),
    };
    settings.save(&path).unwrap();
    let loaded = Settings::load(&path).unwrap();
    assert!(loaded.tidy && loaded.cpplint);
    assert_eq!(loaded.cpplint_path, settings.cpplint_path);
}

#[test]
fn cpp_only_toggles_and_shortcuts_do_not_edit() {
    let dir = Directory::new();
    let mut app = cpp_app(&dir);
    app.toggle_cpp_check(Tool::Tidy);
    assert!(app.cpp_checks.settings.tidy);
    assert!(
        Settings::load(app.cpp_checks.config_path.as_ref().unwrap())
            .unwrap()
            .tidy
    );
    app.toggle_cpp_check(Tool::Tidy);
    let before = app.buffer.editor.get_content();
    for key in ['t', 'l'] {
        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        press(&mut app, KeyCode::Char(key));
        assert!(app.status_bar_text.contains("disabled"));
    }
    assert_eq!(app.buffer.editor.get_content(), before);
    app.buffer = Default::default();
    app.toggle_cpp_check(Tool::Tidy);
    assert!(!app.cpp_checks.settings.tidy);
    app.run_cpp_check(Tool::Tidy);
    assert!(app.status_bar_text.contains("C/C++ only"));
}

#[test]
fn cpplint_prompt_validates_cancels_and_remembers_path() {
    let dir = Directory::new();
    let mut app = cpp_app(&dir);
    let before = app.buffer.editor.get_content();
    app.toggle_cpp_check(Tool::Cpplint);
    assert!(app.cpp_checks.path_input.is_some());
    press(&mut app, KeyCode::Enter);
    assert!(!app.cpp_checks.settings.cpplint);
    press(&mut app, KeyCode::Char('x'));
    press(&mut app, KeyCode::Esc);
    assert!(app.cpp_checks.path_input.is_none());
    assert_eq!(app.buffer.editor.get_content(), before);
    app.toggle_cpp_check(Tool::Cpplint);
    app.cpp_checks.path_input = Some(std::env::current_exe().unwrap().display().to_string());
    press(&mut app, KeyCode::Enter);
    assert!(app.cpp_checks.settings.cpplint);
    assert!(app.cpp_checks.path_input.is_none());
    app.toggle_cpp_check(Tool::Cpplint);
    app.toggle_cpp_check(Tool::Cpplint);
    assert!(app.cpp_checks.settings.cpplint);
    assert!(app.cpp_checks.path_input.is_none());
    let loaded = Settings::load(app.cpp_checks.config_path.as_ref().unwrap()).unwrap();
    assert!(loaded.cpplint);
    assert!(!loaded.cpplint_path.is_empty());
}

#[test]
fn checks_require_saved_current_content() {
    let dir = Directory::new();
    let mut app = cpp_app(&dir);
    app.cpp_checks.settings.tidy = true;
    app.buffer.editor.set_content("changed");
    app.run_cpp_check(Tool::Tidy);
    assert!(app.status_bar_text.contains("Save"));
    assert!(app.cpp_checks.pending.is_none());
    app.buffer.open(dir.0.join("test.cpp")).unwrap();
    std::fs::write(dir.0.join("test.cpp"), "external").unwrap();
    app.run_cpp_check(Tool::Tidy);
    assert!(app.cpp_checks.pending.is_none());
}

#[cfg(unix)]
#[test]
fn checker_runs_literal_paths_and_displays_stderr_and_exit_status() {
    use std::os::unix::fs::PermissionsExt;
    let dir = Directory::new();
    let mut app = cpp_app(&dir);
    let executable = dir.0.join("fake cpplint; literal");
    std::fs::write(
        &executable,
        "#!/bin/sh\nprintf 'checked: %s\\n' \"$1\"\nprintf 'warning: test\\n' >&2\nexit 1\n",
    )
    .unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    app.cpp_checks.settings.cpplint = true;
    app.cpp_checks.settings.cpplint_path = executable.display().to_string();
    app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    press(&mut app, KeyCode::Char('l'));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while app.cpp_checks.pending.is_some() && std::time::Instant::now() < deadline {
        app.poll_cpp_check();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let report = app.cpp_checks.report.as_ref().unwrap();
    assert!(report.contains("warning: test"));
    assert!(report.contains("checked:"));
    assert!(report.contains("exit status: 1"));
    let before = app.buffer.editor.get_content();
    press(&mut app, KeyCode::Char('x'));
    press(&mut app, KeyCode::Down);
    assert!(app.cpp_checks.scroll > 0);
    press(&mut app, KeyCode::Esc);
    assert!(app.cpp_checks.report.is_none());
    assert_eq!(app.buffer.editor.get_content(), before);
}

#[test]
fn preference_write_failure_does_not_enable_tool() {
    let dir = Directory::new();
    let mut app = cpp_app(&dir);
    app.cpp_checks.config_path = Some(dir.0.join("test.cpp/settings.json"));
    app.toggle_cpp_check(Tool::Tidy);
    assert!(!app.cpp_checks.settings.tidy);
    assert!(app.status_bar_text.contains("Cannot save"));
}

#[test]
fn cpp_menu_enters_setup_and_modals_render_on_small_screens() {
    use ratatui::{Terminal, backend::TestBackend};
    let dir = Directory::new();
    let mut app = cpp_app(&dir);
    // F10, then Left wraps to the C/C++ menu, whose second entry is the Cpplint switch.
    press(&mut app, KeyCode::F(10));
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);
    assert!(app.cpp_checks.path_input.is_some());
    for (width, height) in [(1, 1), (10, 3), (80, 24)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        app.cpp_checks.report = Some("test warning\nsecond line".into());
        terminal.draw(|frame| app.draw(frame)).unwrap();
        app.cpp_checks.report = None;
    }
}

#[test]
fn cpplint_shortcut_needs_an_executable_path() {
    let dir = Directory::new();
    let mut app = cpp_app(&dir);
    app.cpp_checks.settings.cpplint = true;
    for path in ["", "/ptre-missing/cpplint", "/"] {
        app.cpp_checks.settings.cpplint_path = path.into();
        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        press(&mut app, KeyCode::Char('l'));
        assert!(
            app.status_bar_text.contains("not an executable"),
            "{path:?}"
        );
        assert!(app.cpp_checks.pending.is_none(), "{path:?}");
    }
}
