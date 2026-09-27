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

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// Echo command chords in Emacs notation for the bottom status bar.
pub(crate) fn shortcut_label(key: KeyEvent, after_prefix: bool) -> Option<String> {
    let modified = key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
    if !modified && !after_prefix && !matches!(key.code, KeyCode::F(_)) {
        return None;
    }
    let code = match key.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::F(number) => format!("F{number}"),
        KeyCode::Enter => "RET".into(),
        KeyCode::Esc => "ESC".into(),
        KeyCode::Tab => "TAB".into(),
        KeyCode::Backspace => "BS".into(),
        KeyCode::Delete => "DEL".into(),
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        KeyCode::Home => "Home".into(),
        KeyCode::End => "End".into(),
        KeyCode::PageUp => "Prior".into(),
        KeyCode::PageDown => "Next".into(),
        _ => return None,
    };
    let mut label = String::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        label.push_str("C-");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        label.push_str("M-");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) && !matches!(key.code, KeyCode::Char(_)) {
        label.push_str("S-");
    }
    label.push_str(&code);
    Some(label)
}
