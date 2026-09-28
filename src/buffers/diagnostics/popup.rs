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

use super::color;
use crate::{app::App, buffers::menu::LIGHT_BLUE, lsp::Diagnostic};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Padding, Paragraph},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const MAX_ROWS: usize = 8;
const MAX_TEXT_WIDTH: usize = 60;

impl App {
    // Diagnostics touching the cursor that describe the current text.
    fn diagnostics_at_cursor(&self) -> Vec<&Diagnostic> {
        let cursor = self.buffer.editor.get_cursor();
        let shown = self.shown_diagnostics().into_iter();
        shown
            .filter(|&(d, fresh)| fresh && d.range.start <= cursor && cursor <= d.range.end)
            .map(|(d, _)| d)
            .collect()
    }

    // Explain the diagnostics under the cursor in a popup below it, like the completion list.
    pub(crate) fn draw_diagnostic_popup(&self, frame: &mut Frame) {
        // Completion, menus and prompts need the space and the user's attention first.
        if self.completion.is_active()
            || self.menu.active.is_some()
            || self.cpp_check_modal()
            || self.save_path_input.is_some()
            || self.overwrite_confirm
        {
            return;
        }
        let at_cursor = self.diagnostics_at_cursor();
        if at_cursor.is_empty() {
            return;
        }
        let Some((x, y)) = self.buffer.editor.get_visible_cursor(&self.editor_area) else {
            return;
        };

        // Borders and one column of padding on each side take four columns.
        let area = frame.area();
        let width = usize::from(area.width.saturating_sub(4)).min(MAX_TEXT_WIDTH);
        let mut lines = Vec::new();
        for diagnostic in at_cursor {
            lines.extend(message_lines(diagnostic, width));
        }
        lines.truncate(MAX_ROWS);

        let text_width = lines.iter().map(Line::width).max().unwrap_or(0) as u16;
        let width = (text_width + 4).min(area.width);
        let height = lines.len() as u16 + 2;
        let x = x.saturating_sub(2).min(area.right().saturating_sub(width));
        let y = if y + 1 + height <= area.bottom() {
            y + 1
        } else {
            y.saturating_sub(height)
        };
        let popup = Rect::new(x, y, width, height).intersection(area);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(LIGHT_BLUE))
            .padding(Padding::horizontal(1));
        frame.render_widget(Clear, popup);
        frame.render_widget(Paragraph::new(lines).block(block), popup);
    }
}

// "error: message [source]" wrapped to `width`, with the severity word in its color.
fn message_lines(diagnostic: &Diagnostic, width: usize) -> Vec<Line<'static>> {
    let label = format!("{}: ", diagnostic.severity.name());
    let source = diagnostic
        .source
        .as_ref()
        .map(|source| format!(" [{source}]"))
        .unwrap_or_default();
    let text = format!("{label}{}{source}", diagnostic.message);
    let base = Style::default().fg(LIGHT_BLUE);
    let label_style = Style::default()
        .fg(color(diagnostic.severity))
        .add_modifier(Modifier::BOLD);

    let mut lines = Vec::new();
    for paragraph in text.lines() {
        for piece in wrap(paragraph, width) {
            // Only the very first piece starts with the label.
            let line = match piece.strip_prefix(&label).filter(|_| lines.is_empty()) {
                Some(rest) => Line::from(vec![
                    Span::styled(label.clone(), label_style),
                    Span::styled(rest.to_owned(), base),
                ]),
                None => Line::from(Span::styled(piece, base)),
            };
            lines.push(line);
        }
    }
    lines
}

// Break `text` into lines at most `width` columns wide, at spaces where possible.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = vec![String::new()];
    for word in text.split(' ') {
        let last = lines.len() - 1;
        if !lines[last].is_empty() {
            if lines[last].width() + 1 + word.width() > width {
                lines.push(String::new());
            } else {
                lines[last].push(' ');
            }
        }
        // A word wider than the popup is cut where it hits the edge.
        for c in word.chars() {
            let last = lines.len() - 1;
            if !lines[last].is_empty() && lines[last].width() + c.width().unwrap_or(0) > width {
                lines.push(String::new());
            }
            let last = lines.len() - 1;
            lines[last].push(c);
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::Severity;
    use ratatui::{Terminal, backend::TestBackend};

    fn screen(app: &mut App) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..24)
            .map(|y| (0..80).map(|x| buffer[(x, y)].symbol()).collect())
            .collect()
    }

    fn app_with_error(fresh: bool) -> App {
        let mut app = App::default();
        app.buffer.editor.set_content("int x = y;\n");
        let error = Diagnostic {
            range: 8..9,
            severity: Severity::Error,
            message: "Use of undeclared identifier 'y'".into(),
            source: Some("clang".into()),
        };
        app.completion.set_diagnostics(vec![error], fresh);
        app
    }

    #[test]
    fn popup_shows_under_the_cursor_only_on_fresh_diagnostics() {
        let expected = "error: Use of undeclared identifier 'y' [clang]";
        for (cursor, fresh, shown) in [(9, true, true), (8, true, true), (2, true, false)]
            .into_iter()
            .chain([(9, false, false)])
        {
            let mut app = app_with_error(fresh);
            app.buffer.editor.set_cursor(cursor);
            let rows = screen(&mut app);
            let (_, y) = app
                .buffer
                .editor
                .get_visible_cursor(&app.editor_area)
                .unwrap();
            let found = rows.iter().position(|row| row.contains(expected));
            assert_eq!(found.is_some(), shown, "cursor {cursor}, fresh {fresh}");
            if shown {
                assert_eq!(
                    found,
                    Some(usize::from(y) + 2),
                    "text is one row below the border"
                );
                assert!(rows[usize::from(y) + 1].contains('╭'));
            }
        }
    }

    #[test]
    fn popup_gives_way_to_menus() {
        let mut app = app_with_error(true);
        app.buffer.editor.set_cursor(9);
        app.menu.active = Some(0);
        assert!(
            !screen(&mut app)
                .iter()
                .any(|row| row.contains("undeclared"))
        );
    }

    #[test]
    fn long_messages_wrap_at_spaces_and_long_words_are_cut() {
        assert_eq!(wrap("aa bb cc", 5), ["aa bb", "cc"]);
        assert_eq!(wrap("abcdefg", 3), ["abc", "def", "g"]);
        assert_eq!(wrap("宽字符测试", 4), ["宽字", "符测", "试"]);
        let diagnostic = Diagnostic {
            range: 0..0,
            severity: Severity::Warning,
            message: "one two three\nnote: four".into(),
            source: None,
        };
        let lines: Vec<String> = message_lines(&diagnostic, 12)
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(lines, ["warning: one", "two three", "note: four"]);
    }
}
