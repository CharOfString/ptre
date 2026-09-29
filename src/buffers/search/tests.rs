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
use ratatui::{Terminal, backend::TestBackend, layout::Rect};

fn app_with(text: &str, cursor: usize) -> App {
    let mut app = App::default();
    app.editor_area = Rect::new(1, 1, 78, 22);
    app.buffer.editor.set_content(text);
    app.buffer.editor.set_cursor(cursor);
    app
}

fn press(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.handle_key(KeyEvent::new(code, modifiers));
}

fn ctrl(app: &mut App, c: char) {
    press(app, KeyCode::Char(c), KeyModifiers::CONTROL);
}

fn type_text(app: &mut App, text: &str) {
    for c in text.chars() {
        let code = if c == '\n' {
            KeyCode::Enter
        } else {
            KeyCode::Char(c)
        };
        press(app, code, KeyModifiers::NONE);
    }
}

fn replace(app: &mut App, from: &str, to: &str) {
    press(app, KeyCode::Char('%'), KeyModifiers::ALT);
    type_text(app, &format!("{from}\n{to}\n"));
}

fn cursor(app: &App) -> usize {
    app.buffer.editor.get_cursor()
}

#[test]
fn matches_ignore_case_only_for_lowercase_queries() {
    assert_eq!(find_all("Foo foo FOO", "foo"), [0..3, 4..7, 8..11]);
    let only_first = Range { start: 0, end: 3 };
    assert_eq!(find_all("Foo foo FOO", "Foo"), [only_first]);
    assert_eq!(find_all("aaaa", "aa"), [0..2, 2..4], "no overlaps");
    assert_eq!(find_all("中文中文", "中"), [0..1, 2..3]);
    assert!(find_all("abc", "").is_empty());
    assert!(find_all("ab", "abc").is_empty());
}

#[test]
fn incremental_search_moves_fails_wraps_and_ends_at_the_match() {
    let mut app = app_with("one two one two", 0);
    ctrl(&mut app, 's');
    type_text(&mut app, "two");
    assert_eq!(cursor(&app), 7, "after the first match");
    assert_eq!(app.search.prompt().as_deref(), Some("I-search: two"));
    assert_eq!(app.buffer.editor.get_marks().map(Vec::len), Some(2));
    assert_eq!(app.buffer.editor.get_content(), "one two one two");

    ctrl(&mut app, 's');
    assert_eq!(cursor(&app), 15);
    ctrl(&mut app, 's');
    assert_eq!(cursor(&app), 15);
    assert_eq!(
        app.search.prompt().as_deref(),
        Some("Failing I-search: two")
    );
    ctrl(&mut app, 's');
    assert_eq!(cursor(&app), 7);
    assert_eq!(
        app.search.prompt().as_deref(),
        Some("Wrapped I-search: two")
    );

    press(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert!(app.search.prompt().is_none());
    assert_eq!(cursor(&app), 7);
    assert!(!app.buffer.editor.has_marks());
}

#[test]
fn incremental_search_backward_cancel_other_keys_and_repeat() {
    let mut app = app_with("one two one two", 15);
    ctrl(&mut app, 'r');
    type_text(&mut app, "one");
    assert_eq!(cursor(&app), 8, "before the nearest match");
    assert_eq!(
        app.search.prompt().as_deref(),
        Some("I-search backward: one")
    );
    ctrl(&mut app, 'g');
    assert_eq!(cursor(&app), 15, "C-g goes back");
    assert!(app.search.prompt().is_none());

    // Another key ends the search and still does its job.
    app.buffer.editor.set_cursor(0);
    ctrl(&mut app, 's');
    type_text(&mut app, "two");
    press(&mut app, KeyCode::Left, KeyModifiers::NONE);
    assert!(app.search.prompt().is_none());
    assert_eq!(cursor(&app), 6);

    // C-s on an empty search repeats the last one; Backspace shortens it.
    ctrl(&mut app, 's');
    ctrl(&mut app, 's');
    assert_eq!(app.search.prompt().as_deref(), Some("I-search: two"));
    assert_eq!(cursor(&app), 15);
    press(&mut app, KeyCode::Backspace, KeyModifiers::NONE);
    assert_eq!(cursor(&app), 14);
    press(&mut app, KeyCode::Enter, KeyModifiers::NONE);

    // A capital letter makes the search case-sensitive.
    let mut app = app_with("foo Foo", 0);
    ctrl(&mut app, 's');
    type_text(&mut app, "F");
    assert_eq!(cursor(&app), 5);
}

#[test]
fn query_replace_asks_at_each_match_and_undoes_in_one_step() {
    let mut app = app_with("a b a b a", 0);
    press(&mut app, KeyCode::Char('%'), KeyModifiers::ALT);
    assert_eq!(app.search.prompt().as_deref(), Some("Query replace: █"));
    type_text(&mut app, "a\n");
    assert_eq!(
        app.search.prompt().as_deref(),
        Some("Query replace a with: █")
    );
    type_text(&mut app, "x\n");
    assert_eq!(
        app.search.prompt().as_deref(),
        Some("Query replacing a with x: (y, n, !, ., q)")
    );
    assert_eq!(cursor(&app), 1);

    type_text(&mut app, "y");
    assert_eq!(app.buffer.editor.get_content(), "x b a b a");
    assert_eq!(cursor(&app), 5, "at the next match");
    type_text(&mut app, "n");
    assert_eq!(cursor(&app), 9);
    type_text(&mut app, "!");
    assert_eq!(app.buffer.editor.get_content(), "x b a b x");
    assert_eq!(app.status_bar_text, "Replaced 2 occurrences");
    assert!(app.search.prompt().is_none());
    assert!(!app.buffer.editor.has_marks());

    ctrl(&mut app, 'z');
    assert_eq!(app.buffer.editor.get_content(), "a b a b a");
}

#[test]
fn query_replace_answers_and_edge_cases() {
    // "." replaces this match and stops.
    let mut app = app_with("cat cat cat", 0);
    replace(&mut app, "cat", "dog");
    type_text(&mut app, ".");
    assert_eq!(app.buffer.editor.get_content(), "dog cat cat");
    assert_eq!(app.status_bar_text, "Replaced 1 occurrence");

    // "q" stops without changes; a replacement holding the text does not loop.
    let mut app = app_with("a a", 0);
    replace(&mut app, "a", "aa");
    type_text(&mut app, "q");
    assert_eq!(app.status_bar_text, "Replaced 0 occurrences");
    // Replacing starts at the cursor, which "q" left after the first match.
    app.buffer.editor.set_cursor(0);
    replace(&mut app, "a", "aa");
    type_text(&mut app, "!");
    assert_eq!(app.buffer.editor.get_content(), "aa aa");

    // Only matches after the cursor count; an empty text cancels.
    let mut app = app_with("a b", 3);
    replace(&mut app, "a", "x");
    assert_eq!(app.status_bar_text, "Replaced 0 occurrences");
    assert!(app.search.prompt().is_none());
    press(
        &mut app,
        KeyCode::Char('%'),
        KeyModifiers::ALT | KeyModifiers::SHIFT,
    );
    type_text(&mut app, "\n");
    assert!(app.search.prompt().is_none());
    assert_eq!(app.buffer.editor.get_content(), "a b");

    // Another key ends replacing and still does its job.
    let mut app = app_with("ab ab", 0);
    replace(&mut app, "ab", "x");
    press(&mut app, KeyCode::Left, KeyModifiers::NONE);
    assert!(app.search.prompt().is_none());
    assert_eq!(cursor(&app), 1);
}

#[test]
fn status_bar_shows_the_prompt() {
    let mut app = app_with("one two", 0);
    ctrl(&mut app, 's');
    type_text(&mut app, "tw");
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let status: String = (0..80)
        .map(|x| terminal.backend().buffer()[(x, 23)].symbol())
        .collect();
    assert!(status.starts_with(" I-search: tw"), "{status}");
}
