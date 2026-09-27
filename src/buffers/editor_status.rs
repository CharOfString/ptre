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

use crate::buffers::menu::LIGHT_BLUE;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};
use ratatui_code_editor::editor::Editor;

// Editor buffer bottom toolbar labels
fn labels(editor: &Editor) -> (String, String) {
    // Where the infos come from.
    let code = editor.code_ref();
    let line = code.char_to_line(editor.get_cursor().min(code.len_chars()));
    let last = code.len_lines().saturating_sub(1);
    let position = if line == 0 {
        "Top".to_owned()
    } else if line == last {
        "Bot".to_owned()
    } else {
        format!("{}%", line * 100 / last)
    };

    // Inspect the first line ending; buffers without one default to LF.
    let first_line = code.line(0);
    let length = first_line.len_chars();
    let ending = if length >= 2
        && first_line.char(length - 2) == '\r'
        && first_line.char(length - 1) == '\n'
    {
        "CRLF"
    } else {
        "LF"
    };

    let language = crate::utils::file_type::display_name(code.lang());
    (
        format!(" L{} {position} · {} chars", line + 1, code.len_chars()),
        format!("{ending} · {language} "),
    )
}

// Draw the actual bar.
pub(super) fn draw(editor: &Editor, frame: &mut Frame, area: Rect, border_area: Rect) {
    if area.height < 2 || area.width == 0 {
        return;
    }

    let separator = match border_area.width {
        0 => String::new(),
        1 => "├".to_owned(),
        width => format!("├{}┤", "─".repeat(usize::from(width - 2))),
    };

    frame.render_widget(
        Paragraph::new(separator).style(Style::default().fg(Color::White)),
        Rect::new(border_area.x, area.y, border_area.width, 1),
    );

    let area = Rect::new(area.x, area.y + 1, area.width, 1);
    let (left, right) = labels(editor);
    let right_width = u16::try_from(right.chars().count()).unwrap_or(u16::MAX);
    let [left_area, right_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(right_width)]).areas(area);
    let style = Style::default().fg(LIGHT_BLUE);
    frame.render_widget(Paragraph::new(left).style(style), left_area);
    frame.render_widget(
        Paragraph::new(right)
            .alignment(Alignment::Right)
            .style(style),
        right_area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use ratatui::{Terminal, backend::TestBackend};
    use ratatui_code_editor::theme::vesper;

    #[test]
    fn reports_line_position_endings_and_language() {
        let mut editor = Editor::new("rust", "a\r\nb\r\nc\r\nd\r\ne", vesper()).unwrap();
        assert_eq!(
            labels(&editor),
            (" L1 Top · 13 chars".into(), "CRLF · Rust ".into())
        );
        editor.set_cursor(editor.code_ref().line_to_char(1));
        assert_eq!(labels(&editor).0, " L2 25% · 13 chars");
        editor.set_cursor(editor.code_ref().len_chars());
        assert_eq!(labels(&editor).0, " L5 Bot · 13 chars");
        editor.set_content("");
        assert_eq!(
            labels(&editor),
            (" L1 Top · 0 chars".into(), "LF · Rust ".into())
        );
        editor.set_content("a\nb");
        assert_eq!(labels(&editor).1, "LF · Rust ");
    }

    #[test]
    fn footer_is_inside_buffer_and_keeps_global_borders() {
        let mut app = App::default();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let screen = terminal.backend().buffer();
        let separator: String = (0..80).map(|x| screen[(x, 19)].symbol()).collect();
        assert!(separator.starts_with("├─"));
        assert!(separator.ends_with("─┤"));
        let footer: String = (0..80).map(|x| screen[(x, 20)].symbol()).collect();
        assert!(footer.starts_with("│ L1 Top · 0 chars"));
        let global: String = (0..80).map(|x| screen[(x, 23)].symbol()).collect();
        assert!(!global.contains("chars"));
        assert!(footer.ends_with("LF · Text │"));
        assert_eq!(app.editor_area.bottom(), 19);
        assert_eq!(screen[(0, 21)].symbol(), "╰");
        assert_eq!(screen[(0, 22)].symbol(), "─");
        assert_eq!(screen[(0, 19)].fg, Color::White);
        assert_eq!(screen[(1, 19)].fg, Color::White);
        assert_eq!(screen[(1, 20)].fg, LIGHT_BLUE);
    }

    #[test]
    fn character_count_updates_and_counts_unicode_characters() {
        let mut editor = Editor::new("text", "中文🙂", vesper()).unwrap();
        assert!(labels(&editor).0.ends_with("3 chars"));
        editor.set_content("你好");
        assert!(labels(&editor).0.ends_with("2 chars"));
    }

    #[test]
    fn small_terminals_do_not_panic() {
        for (width, height) in [(0, 0), (1, 1), (4, 5), (12, 8)] {
            let mut app = App::default();
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
        }
    }
}
