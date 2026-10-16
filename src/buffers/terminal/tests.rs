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

#[test]
fn layout_switches_to_right_in_short_windows() {
    let [editor, terminal] = split(Rect::new(0, 2, 100, 36));
    assert_eq!(terminal.y, editor.bottom());
    assert_eq!(editor.width, terminal.width);
    let [editor, terminal] = split(Rect::new(0, 2, 100, 20));
    assert_eq!(terminal.x, editor.right());
    assert_eq!(editor.height, terminal.height);
    for width in 0..4 {
        for height in 0..4 {
            let _ = split(Rect::new(0, 0, width, height));
        }
    }
}

#[test]
#[cfg(unix)]
fn shell_output_and_resize_work() {
    use ratatui::backend::TestBackend;
    use std::time::{Duration, Instant};
    let mut terminal = Terminal::default();
    terminal.toggle(None).unwrap();
    let session = terminal.session.as_mut().unwrap();
    session
        .writer
        .write_all(b"printf '\\033[31m'; printf 'pty-%s' ready; printf '\\033[0m\\n'\n")
        .unwrap();
    session.writer.flush().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        terminal.poll();
        if terminal
            .session
            .as_ref()
            .unwrap()
            .parser
            .screen()
            .contents()
            .contains("pty-ready")
        {
            break;
        }
        assert!(Instant::now() < deadline, "shell output timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
    let mut display = ratatui::Terminal::new(TestBackend::new(60, 12)).unwrap();
    // Return the top and bottom border rows.
    let borders = |display: &mut ratatui::Terminal<TestBackend>, terminal: &mut Terminal| {
        display
            .draw(|frame| terminal.draw(frame, frame.area()).unwrap())
            .unwrap();
        let screen = display.backend().buffer();
        [0, 11].map(|y| (0..60).map(|x| screen[(x, y)].symbol()).collect::<String>())
    };
    // The shell title replaces the default name once it is set.
    let session = terminal.session.as_mut().unwrap();
    session.parser.callbacks_mut().0.clear();
    let [top, bottom] = borders(&mut display, &mut terminal);
    assert!(top.starts_with("╭─ Terminal ─"));
    assert!(bottom.starts_with("╰──"));
    let session = terminal.session.as_mut().unwrap();
    session.parser.process(b"\x1b]2;make\tbuild\x07");
    assert!(borders(&mut display, &mut terminal)[0].starts_with("╭─ makebuild ─"));
    assert_eq!(
        terminal
            .session
            .as_ref()
            .unwrap()
            .master
            .get_size()
            .unwrap()
            .cols,
        58
    );
    terminal.toggle(None).unwrap();
    assert!(!terminal.visible);
    assert!(terminal.session.is_some());
}

#[test]
fn status_bar_shows_keys_of_focused_pane() {
    let mut app = crate::app::App::default();
    let mut display = ratatui::Terminal::new(ratatui::backend::TestBackend::new(100, 24)).unwrap();
    let mut status = |app: &mut crate::app::App| {
        display.draw(|frame| app.draw(frame)).unwrap();
        let screen = display.backend().buffer();
        (0..100)
            .map(|x| screen[(x, 23)].symbol())
            .collect::<String>()
    };
    app.terminal.visible = true;
    assert!(status(&mut app).contains("F10: menu  C-c t t: terminal"));
    app.terminal.focused = true;
    let text = status(&mut app);
    assert!(text.contains("C-x o: focus  M-v/C-v: scroll"));
    assert!(!text.contains("F10"));
}

#[test]
fn keys_preserve_shell_controls() {
    assert_eq!(
        encode_key(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            false
        ),
        [3]
    );
    assert_eq!(
        encode_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), true),
        b"\x1bOA"
    );
    assert_eq!(
        encode_key(
            KeyEvent::new(KeyCode::Char('中'), KeyModifiers::NONE),
            false
        ),
        "中".as_bytes()
    );
}

#[test]
fn terminal_focus_does_not_edit_buffer() {
    let mut app = crate::app::App::default();
    app.terminal.visible = true;
    app.terminal.focused = true;
    app.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
    assert_eq!(app.buffer.editor.get_content(), "");
    app.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL));
    app.handle_key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE));
    assert!(!app.terminal.focused);
    assert!(!app.ctrl_x_wait_flag);
}

#[test]
fn history_pages_clamp_and_preserve_output() {
    let mut parser = vt100::Parser::new(4, 20, 10);
    for n in 0..20 {
        parser.process(format!("line {n}\r\n").as_bytes());
    }
    let live = parser.screen().contents();
    let up = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::ALT);
    let down = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL);
    assert!(scroll_history(parser.screen_mut(), up));
    assert_eq!(parser.screen().scrollback(), 3);
    assert_ne!(parser.screen().contents(), live);
    let history = parser.screen().contents();
    parser.process(b"new line\r\n");
    assert_eq!(parser.screen().contents(), history);
    for _ in 0..20 {
        scroll_history(
            parser.screen_mut(),
            KeyEvent::new(KeyCode::PageUp, KeyModifiers::SHIFT),
        );
    }
    assert_eq!(parser.screen().scrollback(), 10);
    assert!(scroll_history(parser.screen_mut(), down));
    assert_eq!(parser.screen().scrollback(), 7);
    for _ in 0..20 {
        scroll_history(
            parser.screen_mut(),
            KeyEvent::new(KeyCode::PageDown, KeyModifiers::SHIFT),
        );
    }
    assert_eq!(parser.screen().scrollback(), 0);
    assert!(parser.screen().contents().contains("new line"));
    assert!(!scroll_history(
        parser.screen_mut(),
        KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)
    ));
    parser.process(b"\x1b[?1049h");
    assert!(!scroll_history(parser.screen_mut(), up));
    assert!(!scroll_history(parser.screen_mut(), down));
}

#[test]
#[cfg(unix)]
fn history_keys_are_local_and_input_returns_to_live_output() {
    let mut app = crate::app::App::default();
    app.terminal.toggle(None).unwrap();
    let session = app.terminal.session.as_mut().unwrap();
    for _ in 0..40 {
        session.parser.process(b"history\r\n");
    }
    // An exited shell still allows browsing its output.
    session.exited = true;
    app.handle_key(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::ALT));
    assert!(
        app.terminal
            .session
            .as_ref()
            .unwrap()
            .parser
            .screen()
            .scrollback()
            > 0
    );
    app.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
    assert_eq!(
        app.terminal
            .session
            .as_ref()
            .unwrap()
            .parser
            .screen()
            .scrollback(),
        0
    );
    assert_eq!(app.buffer.editor.get_content(), "");
}
