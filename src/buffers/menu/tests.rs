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
use ratatui::{Terminal, backend::TestBackend};

#[test]
fn cpp_menu_hides_checks_that_cannot_run() {
    let mut settings = Settings::default();
    assert_eq!(menus(false), ["FILE", "EDIT", "LSP", "WINDOW"]);
    assert_eq!(menus(true), ["FILE", "EDIT", "LSP", "WINDOW", "C/C++"]);
    assert_eq!(items(2, true, false, Some(&settings)).len(), 3);
    let labels = |settings: &Settings| {
        let entries = items(CPP_MENU, true, false, Some(settings));
        entries
            .into_iter()
            .map(|(label, _)| label)
            .collect::<Vec<_>>()
    };
    assert_eq!(labels(&settings), ["Clang-tidy: Off", "Cpplint: Off"]);
    settings.tidy = true;
    assert_eq!(
        labels(&settings),
        ["Clang-tidy: On", "Cpplint: Off", "Run Clang-tidy"]
    );

    // Cpplint also needs an executable file as its path.
    settings.cpplint = true;
    for path in ["", "/ptre-missing/cpplint", "/", "Cargo.toml"] {
        settings.cpplint_path = path.into();
        assert_eq!(labels(&settings).len(), 3, "{path:?}");
    }
    settings.cpplint_path = std::env::current_exe().unwrap().display().to_string();
    assert_eq!(
        items(CPP_MENU, true, false, Some(&settings))[3],
        ("Run Cpplint", "C-c l")
    );
    assert!(items(CPP_MENU, true, false, None).is_empty());
    assert_eq!(items(0, true, false, Some(&settings)), FILE);
}

#[test]
fn cpp_menu_shows_only_for_c_and_cpp_buffers() {
    let dir = std::env::temp_dir().join(format!("ptre-menu-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let bar = |app: &mut App| {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let screen = terminal.backend().buffer();
        (0..80).map(|x| screen[(x, 0)].symbol()).collect::<String>()
    };
    for (file, cpp) in [("main.c", true), ("main.cpp", true), ("notes.txt", false)] {
        let path = dir.join(file);
        std::fs::write(&path, "text").unwrap();
        let mut app = App::default();
        app.buffer.open(path).unwrap();
        assert_eq!(bar(&mut app).contains("C/C++"), cpp, "{file}");

        // Left from FILE wraps to the last menu shown.
        app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        assert_eq!(
            app.menu.active,
            Some(if cpp { CPP_MENU } else { WINDOW_MENU })
        );
    }

    // With "Run Clang-tidy" hidden, the third entry runs Cpplint.
    let mut app = App::default();
    app.buffer.open(dir.join("main.c")).unwrap();
    let settings = &mut app.cpp_checks.settings;
    settings.cpplint = true;
    settings.cpplint_path = std::env::current_exe().unwrap().display().to_string();
    app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(app.status_bar_text, "Running Cpplint…");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn renders_light_blue_menu_and_separator() {
    let mut app = App::default();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let screen = terminal.backend().buffer();
    let title: String = (0..80).map(|x| screen[(x, 0)].symbol()).collect();
    assert!(title.contains(&format!("PTRE REL.{}", env!("CARGO_PKG_VERSION"))));
    assert!(title.contains("FILE"));
    assert!(title.contains("EDIT"));
    assert_eq!(screen[(0, 0)].bg, Color::Reset);
    assert_eq!(screen[(2, 0)].fg, LIGHT_BLUE);
    assert_eq!(screen[(0, 1)].symbol(), "╭");
    assert_eq!(screen[(79, 1)].symbol(), "╮");
    for x in 1..79 {
        assert_eq!(screen[(x, 1)].symbol(), "─");
    }
    for x in 0..80 {
        assert!(!screen[(x, 0)].modifier.contains(Modifier::UNDERLINED));
        assert_eq!(screen[(x, 1)].fg, LIGHT_BLUE);
        assert_eq!(screen[(x, 1)].bg, Color::Reset);
    }
    app.menu.active = Some(0);
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let screen = terminal.backend().buffer();
    let popup_x = 10 + env!("CARGO_PKG_VERSION").len() as u16;
    assert_eq!(screen[(popup_x, 1)].bg, Color::Reset);
    assert_eq!(screen[(popup_x, 1)].fg, LIGHT_BLUE);
}

#[test]
fn active_menu_and_popup_use_markers_without_underlines() {
    for active in 0..3 {
        let mut app = App::default();
        app.menu.active = Some(active);
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let screen = terminal.backend().buffer();
        let popup_x = 10 + env!("CARGO_PKG_VERSION").len() as u16 + active as u16 * 8;
        let label: String = (0..80).map(|x| screen[(x, 0)].symbol()).collect();
        assert!(label.contains(&format!("❃ {}", MENUS[active])));
        assert_eq!(screen[(popup_x, 1)].symbol(), "╭");
        let popup_width = [18, 17, 31][active];
        assert_eq!(screen[(popup_x + popup_width - 1, 1)].symbol(), "╮");
        assert_eq!(screen[(popup_x + 1, 2)].symbol(), "·");
        assert_eq!(screen[(popup_x + 1, 3)].symbol(), " ");
        for y in 0..4 {
            for x in 0..80 {
                let cell = &screen[(x, y)];
                assert!(
                    !cell
                        .modifier
                        .intersects(Modifier::REVERSED | Modifier::UNDERLINED)
                );
                assert_eq!(cell.bg, Color::Reset);
            }
        }
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let screen = terminal.backend().buffer();
        assert_eq!(screen[(popup_x + 1, 2)].symbol(), " ");
        assert_eq!(screen[(popup_x + 1, 3)].symbol(), "·");
    }
}

#[test]
fn menu_consumes_typing_and_save_opens_prompt() {
    let mut app = App::default();
    app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
    assert_eq!(app.buffer.editor.get_content(), "");
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(app.save_path_input.is_some());
    assert!(app.menu.active.is_none());
}

#[test]
fn edit_menu_undo_and_escape() {
    let mut app = App::default();
    app.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::ALT));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(app.buffer.editor.get_content(), "");
    app.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::ALT));
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(app.buffer.editor.get_content(), "a");
    app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
    assert_eq!(app.menu.active, Some(1));
    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(app.menu.active.is_none());
}

#[test]
fn lsp_menu_toggles_auto_completion_and_opens_popup() {
    let mut app = App::default();
    app.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::ALT));
    app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
    assert_eq!(app.menu.active, Some(2));
    app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
    assert_eq!(app.menu.active, Some(WINDOW_MENU));
    app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
    assert_eq!(app.menu.active, Some(2));
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let row = |app: &mut App, terminal: &mut Terminal<TestBackend>| {
        terminal.draw(|frame| app.draw(frame)).unwrap();
        (0..80)
            .map(|x| terminal.backend().buffer()[(x, 2)].symbol().to_owned())
            .collect::<String>()
    };
    assert!(row(&mut app, &mut terminal).contains("Auto Complete: On  C-c M-l"));

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(!app.completion.auto_enabled());
    assert!(app.menu.active.is_none());
    // FILE, EDIT, LSP from the left.
    app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
    assert!(row(&mut app, &mut terminal).contains("Auto Complete: Off  C-c M-l"));

    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    for c in "hello he".chars() {
        app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }
    app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(app.buffer.editor.get_content(), "hello hello");
}

#[test]
fn tiny_terminals_do_not_panic() {
    for (width, height) in [(1, 1), (5, 2), (20, 4)] {
        let mut app = App::default();
        app.menu.active = Some(2);
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
    }
}

#[test]
fn window_menu_toggles_nerd_font_icons() {
    let mut app = App::default();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let screen = |app: &mut App, terminal: &mut Terminal<TestBackend>| {
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..3)
            .map(|y| (0..80).map(|x| buffer[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
    };

    // In a text buffer, Left from FILE reaches WINDOW, the last menu.
    app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
    assert_eq!(app.menu.active, Some(WINDOW_MENU));
    let rows = screen(&mut app, &mut terminal);
    assert!(rows[0].contains("❃ WINDOW"));
    assert!(rows[2].contains("NF Mode: Off"));

    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(app.nerd_font);
    assert!(app.status_bar_text.is_empty(), "no status message");
    app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
    assert!(screen(&mut app, &mut terminal)[2].contains("NF Mode: On"));
    app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(!app.nerd_font);
}
