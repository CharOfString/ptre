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

const NONE: KeyModifiers = KeyModifiers::NONE;

fn key(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(code, NONE));
}

// C-x and then `code`.
fn chord(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
    key(app, code);
}

// The tab names in order; the active one starts with "*".
fn tabs(app: &App) -> Vec<String> {
    let active = app.tabs.left.len();
    let names = app.tab_buffers().map(|buffer| match buffer.is_dirty() {
        true => format!("{} ●", name(buffer)),
        false => name(buffer),
    });
    names
        .enumerate()
        .map(|(index, name)| {
            if index == active {
                format!("*{name}")
            } else {
                name
            }
        })
        .collect()
}

#[test]
fn new_tabs_follow_the_active_one_and_open_files_are_reused() {
    let mut app = App::default();
    // The first file takes the place of the empty unnamed tab.
    app.open_tab("Cargo.toml".into()).unwrap();
    assert_eq!(tabs(&app), ["*Cargo.toml"]);
    app.open_tab("README.md".into()).unwrap();
    chord(&mut app, KeyCode::Left);
    app.open_tab("src/main.rs".into()).unwrap();
    assert_eq!(tabs(&app), ["Cargo.toml", "*main.rs", "README.md"]);

    // An open file gets its tab back, even with another form of its path.
    app.open_tab(std::path::absolute("README.md").unwrap())
        .unwrap();
    assert_eq!(tabs(&app), ["Cargo.toml", "main.rs", "*README.md"]);
    assert!(app.open_tab("missing-file".into()).is_err());
    assert_eq!(tabs(&app).len(), 3);

    // C-x Right and C-x Left go round at the ends.
    chord(&mut app, KeyCode::Right);
    assert_eq!(tabs(&app)[0], "*Cargo.toml");
    chord(&mut app, KeyCode::Left);
    assert_eq!(tabs(&app)[2], "*README.md");
    assert!(!app.ctrl_x_wait_flag);
}

#[test]
fn closing_a_tab_with_changes_asks_first() {
    let mut app = App::default();
    app.open_tab("Cargo.toml".into()).unwrap();
    app.open_tab("README.md".into()).unwrap();
    app.buffer.editor.set_content("changed");
    chord(&mut app, KeyCode::Char('k'));
    assert!(app.tabs.close_confirm);

    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let screen = terminal.backend().buffer();
    let text: String = screen.content.iter().map(|cell| cell.symbol()).collect();
    assert!(text.contains("Buffer has unsaved changes. Close it?"));

    // Other keys do not edit while the dialog is open; N keeps the tab.
    key(&mut app, KeyCode::Char('x'));
    key(&mut app, KeyCode::Char('n'));
    assert_eq!(app.buffer.editor.get_content(), "changed");
    assert_eq!(tabs(&app), ["Cargo.toml", "*README.md ●"]);

    chord(&mut app, KeyCode::Char('k'));
    key(&mut app, KeyCode::Char('y'));
    assert_eq!(tabs(&app), ["*Cargo.toml"]);
    // A tab without changes closes at once; closing the last one leaves an empty tab.
    chord(&mut app, KeyCode::Char('k'));
    assert_eq!(tabs(&app), ["*[No Name]"]);
    assert!(app.buffer.path.is_none());
}

// Tab row
fn row(app: &App, width: u16) -> (String, ratatui::buffer::Buffer) {
    let mut terminal = Terminal::new(TestBackend::new(width, 1)).unwrap();
    terminal
        .draw(|frame| app.draw_tabs(frame, frame.area()))
        .unwrap();
    let screen = terminal.backend().buffer().clone();
    let row = (0..width).map(|x| screen[(x, 0)].symbol()).collect();
    (row, screen)
}

#[test]
fn tab_row_keeps_the_active_tab_in_view() {
    let mut app = App::default();
    for path in ["Cargo.toml", "README.md", "src/main.rs", "src/app.rs"] {
        app.open_tab(path.into()).unwrap();
    }
    app.buffer.editor.set_content("changed");
    let (row, screen) = row(&app, 30);
    assert!(row.starts_with(" ← main.rs  ❃ app.rs ●  "), "{row}");
    // The active tab is bold light blue, the others are dimmer, and the dot is amber.
    assert_eq!(screen[(14, 0)].fg, LIGHT_BLUE);
    assert!(screen[(14, 0)].modifier.contains(Modifier::BOLD));
    assert_eq!(screen[(1, 0)].fg, MUTED);
    assert_eq!(screen[(3, 0)].fg, MUTED);
    assert_eq!(screen[(21, 0)].fg, AMBER);
    assert!(screen.content.iter().all(|cell| cell.bg == Color::Reset));
}

#[test]
fn arrows_mark_hidden_tabs_and_padding_surrounds_the_row() {
    let mut app = App::default();
    for path in ["Cargo.toml", "README.md", "src/main.rs", "src/app.rs"] {
        app.open_tab(path.into()).unwrap();
    }
    // All tabs fit: only the padding is added.
    assert_eq!(
        row(&app, 50).0,
        "   Cargo.toml    README.md    main.rs  ❃ app.rs   "
    );
    chord(&mut app, KeyCode::Left);
    chord(&mut app, KeyCode::Left);
    assert_eq!(row(&app, 30).0, "   Cargo.toml  ❃ README.md →  ");
    // "← " needs its own room before the active tab.
    chord(&mut app, KeyCode::Right);
    assert_eq!(row(&app, 20).0, " ← ❃ main.rs →      ");
}
