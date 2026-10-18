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
use std::sync::atomic::{AtomicUsize, Ordering};

const NONE: KeyModifiers = KeyModifiers::NONE;
const CONTROL: KeyModifiers = KeyModifiers::CONTROL;

// A small project in the temporary folder, removed after the test.
struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "ptre-sidebar-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        for dir in [".git", "src/inner"] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        for file in [".git/config", ".hidden", "a.txt", "B.txt", "src/main.rs"] {
            fs::write(root.join(file), file).unwrap();
        }
        Self(root)
    }

    // An app with src/main.rs open and the tree shown by C-c c t.
    fn app(&self) -> App {
        let mut app = App::default();
        app.buffer.open(self.0.join("src/main.rs")).unwrap();
        toggle(&mut app);
        app
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.handle_key(KeyEvent::new(code, modifiers));
}

fn toggle(app: &mut App) {
    key(app, KeyCode::Char('c'), CONTROL);
    key(app, KeyCode::Char('c'), NONE);
    key(app, KeyCode::Char('t'), NONE);
}

// The rows as they are indented on screen.
fn rows(app: &App) -> Vec<String> {
    let rows = app.sidebar.rows.iter();
    rows.map(|row| format!("{}{}", "  ".repeat(row.depth), name(&row.path)))
        .collect()
}

fn selected(app: &App) -> String {
    name(&app.sidebar.selected().unwrap().path)
}

#[test]
fn tree_opens_at_the_project_and_the_open_file() {
    let project = Project::new();
    let mut app = project.app();
    assert!(app.sidebar.visible && app.sidebar.focused);
    assert_eq!(app.sidebar.root, project.0);
    // Folders come first; .git is left out.
    assert_eq!(
        rows(&app),
        ["src", "  inner", "  main.rs", ".hidden", "a.txt", "B.txt"]
    );
    assert_eq!(selected(&app), "main.rs");

    // Typing in the tree does not change the buffer.
    key(&mut app, KeyCode::Char('x'), NONE);
    assert_eq!(app.buffer.editor.get_content(), "src/main.rs");
    app.handle_paste("pasted".into());
    assert_eq!(app.buffer.editor.get_content(), "src/main.rs");

    toggle(&mut app);
    assert!(!app.sidebar.visible && !app.sidebar.focused);
}

#[test]
fn keys_move_fold_and_find_the_parent() {
    let project = Project::new();
    let mut app = project.app();
    key(&mut app, KeyCode::Char('p'), NONE);
    assert_eq!(selected(&app), "inner");
    key(&mut app, KeyCode::Char('u'), NONE);
    assert_eq!(selected(&app), "src");

    // TAB closes the folder, and opens it again.
    key(&mut app, KeyCode::Tab, NONE);
    assert_eq!(rows(&app), ["src", ".hidden", "a.txt", "B.txt"]);
    key(&mut app, KeyCode::Enter, NONE);
    assert_eq!(rows(&app).len(), 6);
    assert_eq!(selected(&app), "src");

    // The selection stops at both ends.
    key(&mut app, KeyCode::Up, NONE);
    assert_eq!(selected(&app), "src");
    for _ in 0..10 {
        key(&mut app, KeyCode::Char('n'), CONTROL);
    }
    assert_eq!(selected(&app), "B.txt");

    // A new file shows after g.
    fs::write(project.0.join("c.txt"), "").unwrap();
    key(&mut app, KeyCode::Char('g'), NONE);
    assert_eq!(rows(&app).last().unwrap(), "c.txt");
    assert_eq!(selected(&app), "B.txt");
}

#[test]
fn enter_opens_files_in_tabs() {
    let project = Project::new();
    let mut app = project.app();
    key(&mut app, KeyCode::Down, NONE);
    key(&mut app, KeyCode::Down, NONE);
    key(&mut app, KeyCode::Enter, NONE);
    assert_eq!(app.buffer.path, Some(project.0.join("a.txt")));
    assert_eq!(app.buffer.editor.get_content(), "a.txt");
    assert!(app.sidebar.visible && !app.sidebar.focused);

    // C-x o goes back to the tree.
    key(&mut app, KeyCode::Char('x'), CONTROL);
    key(&mut app, KeyCode::Char('o'), NONE);
    assert!(app.sidebar.focused);

    // Unsaved changes stay in their own tab when another file opens.
    app.buffer.editor.set_content("changed");
    key(&mut app, KeyCode::Down, NONE);
    key(&mut app, KeyCode::Enter, NONE);
    assert_eq!(app.buffer.path, Some(project.0.join("B.txt")));
    key(&mut app, KeyCode::Char('x'), CONTROL);
    key(&mut app, KeyCode::Left, NONE);
    assert_eq!(app.buffer.path, Some(project.0.join("a.txt")));
    assert_eq!(app.buffer.editor.get_content(), "changed");
}

#[test]
fn revealed_file_is_drawn_in_the_middle() {
    let project = Project::new();
    for n in 0..30 {
        fs::write(project.0.join(format!("f{n:02}.txt")), "").unwrap();
    }
    let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
    // Rows: src, .hidden, a.txt, B.txt, then f00.txt to f29.txt. The tree shows 18 rows.
    for (file, offset) in [("f15.txt", 10), ("f29.txt", 16)] {
        let mut app = App::default();
        app.buffer.open(project.0.join(file)).unwrap();
        toggle(&mut app);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        // Near the end, the last row stays at the bottom.
        assert_eq!(app.sidebar.list.offset(), offset, "{file}");
    }
}

#[test]
fn ctrl_c_ctrl_p_a_opens_a_project_that_stays() {
    let project = Project::new();
    let src = project.0.join("src").canonicalize().unwrap();
    let mut app = App::default();
    app.buffer.open(project.0.join("a.txt")).unwrap();
    let open_project = |app: &mut App| {
        key(app, KeyCode::Char('c'), CONTROL);
        key(app, KeyCode::Char('p'), CONTROL);
        key(app, KeyCode::Char('a'), NONE);
        assert!(app.sidebar.project_input.is_some());
    };
    open_project(&mut app);
    key(&mut app, KeyCode::Esc, NONE);
    assert_eq!(app.status_bar_text, "Open project cancelled");

    // A missing folder keeps the prompt open; typing never reaches the buffer.
    open_project(&mut app);
    app.handle_paste("/missing".into());
    key(&mut app, KeyCode::Enter, NONE);
    assert_eq!(app.status_bar_text, "No such folder");
    for _ in 0.."/missing".len() {
        key(&mut app, KeyCode::Backspace, NONE);
    }
    app.handle_paste(src.display().to_string());
    key(&mut app, KeyCode::Enter, NONE);
    assert_eq!(app.buffer.editor.get_content(), "a.txt");
    assert!(app.sidebar.project_input.is_none() && app.sidebar.focused);
    assert_eq!(app.sidebar.root, src);
    assert_eq!(rows(&app), ["inner", "main.rs"]);

    // The tree keeps the project while the open file is inside it.
    key(&mut app, KeyCode::Down, NONE);
    key(&mut app, KeyCode::Enter, NONE);
    toggle(&mut app);
    toggle(&mut app);
    assert_eq!(app.sidebar.root, src);
    assert_eq!(selected(&app), "main.rs");

    // Back in a.txt, it starts at the project of a.txt again.
    key(&mut app, KeyCode::Char('x'), CONTROL);
    key(&mut app, KeyCode::Left, NONE);
    toggle(&mut app);
    toggle(&mut app);
    assert_eq!(app.sidebar.root, project.0);
    assert_eq!(selected(&app), "a.txt");
}

#[test]
fn files_below_the_working_directory_open_with_a_short_path() {
    let mut app = App::default();
    app.buffer.open(PathBuf::from("src/main.rs")).unwrap();
    toggle(&mut app);
    let rows = &app.sidebar.rows;
    let index = rows.iter().position(|row| name(&row.path) == "Cargo.toml");
    app.sidebar.list.select(index);
    key(&mut app, KeyCode::Enter, NONE);
    assert_eq!(app.buffer.path, Some(PathBuf::from("Cargo.toml")));
}

#[test]
fn focus_moves_through_tree_editor_and_terminal() {
    let project = Project::new();
    let mut app = project.app();
    app.terminal.visible = true;
    let mut panes = Vec::new();
    for _ in 0..3 {
        key(&mut app, KeyCode::Char('x'), CONTROL);
        key(&mut app, KeyCode::Char('o'), NONE);
        panes.push((app.sidebar.focused, app.terminal.focused));
    }
    assert_eq!(panes, [(false, false), (false, true), (true, false)]);

    // Hiding the tree gives focus back to the editor.
    key(&mut app, KeyCode::Char('q'), NONE);
    assert!(!app.sidebar.visible && !app.sidebar.focused && !app.terminal.focused);
}

#[test]
fn tree_is_drawn_on_the_left_and_window_menu_toggles_it() {
    let project = Project::new();
    let mut app = project.app();
    let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let screen = terminal.backend().buffer();
    let line = |y: u16| {
        (0..100)
            .map(|x| screen[(x, y)].symbol())
            .collect::<String>()
    };
    assert!(line(2).starts_with("╭─ ptre-sidebar-test-"));
    assert!(line(3).starts_with("│▾ src"));
    assert!(line(4).starts_with("│  ▸ inner"));
    assert!(line(5).starts_with("│    main.rs"));
    // The selected row has a soft bar, not reversed colors.
    assert_eq!(screen[(5, 5)].bg, SELECTED);
    assert!(!screen[(5, 5)].modifier.contains(Modifier::REVERSED));
    assert!(line(23).contains("RET: open"));
    assert_eq!(app.editor_area.x, 31);

    // WINDOW is the last menu in a text buffer; its second entry is the tree.
    key(&mut app, KeyCode::F(10), NONE);
    key(&mut app, KeyCode::Left, NONE);
    key(&mut app, KeyCode::Down, NONE);
    key(&mut app, KeyCode::Enter, NONE);
    assert!(!app.sidebar.visible);
}
