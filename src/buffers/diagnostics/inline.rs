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
use crate::{app::App, lsp::Diagnostic};
use ratatui::{
    Frame,
    style::{Modifier, Style},
};

impl App {
    // Mark diagnostics on the visible lines: a dot in the line number column, an underline
    // under the marked text and the most severe message after the end of the line.
    pub(crate) fn draw_diagnostics(&self, frame: &mut Frame) {
        let shown = self.shown_diagnostics();
        let diagnostics: Vec<&Diagnostic> = shown.into_iter().map(|(d, _)| d).collect();
        if diagnostics.is_empty() {
            return;
        }
        let editor = &self.buffer.editor;
        let code = editor.code_ref();
        let area = self.editor_area;
        let buf = frame.buffer_mut();
        let mut previous_line = None;
        for y in area.top()..area.bottom() {
            // The first text cell tells which line a row shows. Rows past the end of the file
            // repeat the last line, so only its first row counts.
            let Some((text_x, first)) = (area.left()..area.right()).find_map(|x| {
                editor
                    .cursor_from_mouse(x, y, &area)
                    .map(|offset| (x, offset))
            }) else {
                continue;
            };
            let line = code.char_to_line(first);
            if previous_line.replace(line) == Some(line) {
                continue;
            }
            let start = code.line_to_char(line);
            let end = start + code.line_len(line);
            let on_line: Vec<&Diagnostic> = diagnostics
                .iter()
                .copied()
                .filter(|d| d.range.start <= end && d.range.end.max(d.range.start + 1) > start)
                .collect();
            let Some(worst) = on_line.iter().map(|d| d.severity).min() else {
                continue;
            };
            buf[(area.left(), y)].set_symbol("●").set_fg(color(worst));

            // Underline marked chars, up to the first cell after the text of the line.
            let mut end_x = None;
            for x in text_x..area.right() {
                let Some(offset) = editor.cursor_from_mouse(x, y, &area) else {
                    break;
                };
                if offset >= end {
                    end_x = Some(x);
                    break;
                }
                let marked = on_line.iter().filter(|d| d.marks(offset));
                if let Some(severity) = marked.map(|d| d.severity).min() {
                    let cell = &mut buf[(x, y)];
                    cell.modifier.insert(Modifier::UNDERLINED);
                    cell.underline_color = color(severity);
                }
            }

            // The message of the most severe diagnostic starting on this line, if it fits.
            let starting = on_line.iter().filter(|d| d.range.start >= start);
            if let Some(diagnostic) = starting.min_by_key(|d| d.severity)
                && let Some(x) = end_x.map(|x| x + 2).filter(|&x| x < area.right())
            {
                let message = diagnostic.message.lines().next().unwrap_or_default();
                let style = Style::default()
                    .fg(color(diagnostic.severity))
                    .add_modifier(Modifier::ITALIC);
                buf.set_stringn(x, y, message, usize::from(area.right() - x), style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::Severity;
    use ratatui::{Terminal, backend::TestBackend, style::Color};

    fn diagnostic(range: std::ops::Range<usize>, severity: Severity, message: &str) -> Diagnostic {
        Diagnostic {
            range,
            severity,
            message: message.into(),
            source: None,
        }
    }

    #[test]
    fn marks_dot_underline_and_message_on_the_right_rows() {
        let mut app = App::default();
        app.buffer.editor.set_content("int x = y;\nint ok;\nz");
        app.completion.set_diagnostics(
            vec![
                diagnostic(
                    8..9,
                    Severity::Error,
                    "Use of undeclared identifier 'y'\nnote",
                ),
                diagnostic(9..9, Severity::Warning, "expected ';'"),
                diagnostic(19..20, Severity::Warning, "unused 'z'"),
            ],
            false,
        );
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let screen = terminal.backend().buffer();
        let area = app.editor_area;
        let row = |y: u16| (0..80).map(|x| screen[(x, y)].symbol()).collect::<String>();
        let (x_y, y) = (0..80)
            .flat_map(|x| (0..24).map(move |y| (x, y)))
            .find(|&(x, y)| screen[(x, y)].symbol() == "y")
            .unwrap();

        // Line 1: error dot, red underline under `y` only, first message line after the text.
        assert_eq!(screen[(area.left(), y)].symbol(), "●");
        assert_eq!(screen[(area.left(), y)].fg, Color::LightRed);
        assert!(screen[(x_y, y)].modifier.contains(Modifier::UNDERLINED));
        assert_eq!(screen[(x_y, y)].underline_color, Color::LightRed);
        assert!(!screen[(x_y - 1, y)].modifier.contains(Modifier::UNDERLINED));
        assert!(row(y).contains("int x = y;  Use of undeclared identifier 'y'"));
        assert!(!row(y).contains("note"));

        // Line 2 is clean; line 3 has a warning; rows past the end repeat nothing.
        assert_eq!(screen[(area.left(), y + 1)].symbol(), " ");
        assert_eq!(screen[(area.left(), y + 2)].fg, Color::Yellow);
        assert!(row(y + 2).contains("z  unused 'z'"));
        assert_eq!(screen[(area.left(), y + 3)].symbol(), " ");
        assert!(!row(y + 3).contains("unused"));
    }
}
