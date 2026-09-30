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

// Diagnostics in the editor: marks on the visible lines (`inline`) and a popup for the
// diagnostics under the cursor (`popup`). They come from the language server and Cpplint.

mod inline;
mod popup;

use crate::{
    app::App,
    buffers::menu::LIGHT_BLUE,
    lsp::{Diagnostic, Severity},
};
use ratatui::style::Color;

impl App {
    // All diagnostics to show, each with whether it describes the text as it is now.
    // Cpplint findings are only returned while they do.
    fn shown_diagnostics(&self) -> Vec<(&Diagnostic, bool)> {
        let fresh = self.completion.diagnostics_fresh();
        let server = self.completion.diagnostics().iter().map(|d| (d, fresh));
        let cpplint = self.cpplint_diagnostics().iter().map(|d| (d, true));
        server.chain(cpplint).collect()
    }

    // Numbers of errors and warnings in the buffer.
    pub(crate) fn diagnostic_counts(&self) -> (usize, usize) {
        let shown = self.shown_diagnostics();
        let count = |severity| shown.iter().filter(|(d, _)| d.severity == severity).count();
        (count(Severity::Error), count(Severity::Warning))
    }
}

// Coloring config for the status bar.
pub(super) fn nerd_icon(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "\u{f057}",
        Severity::Warning => "\u{f071}",
        Severity::Information => "\u{f05a}",
        Severity::Hint => "\u{f0eb}",
    }
}

pub(super) fn color(severity: Severity) -> Color {
    match severity {
        Severity::Error => Color::LightRed,
        Severity::Warning => Color::Yellow,
        Severity::Information => LIGHT_BLUE,
        Severity::Hint => Color::Gray,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn status_line_counts_errors_and_warnings_only() {
        let mut app = App::default();
        let diagnostic = |severity| Diagnostic {
            range: 0..0,
            severity,
            message: String::new(),
            source: None,
        };
        let severities = [
            Severity::Error,
            Severity::Warning,
            Severity::Error,
            Severity::Hint,
            Severity::Information,
        ];
        app.completion
            .set_diagnostics(severities.map(diagnostic).to_vec(), false);
        assert_eq!(app.diagnostic_counts(), (2, 1));
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let footer: String = (0..80)
            .map(|x| terminal.backend().buffer()[(x, 20)].symbol())
            .collect();
        assert!(footer.ends_with("E: 2 · W: 1 · LF · Text │"), "{footer}");
        let x = footer.chars().position(|c| c == 'E').unwrap() as u16;
        let screen = terminal.backend().buffer();
        assert_eq!(screen[(x, 20)].fg, Color::LightRed, "E");
        assert_eq!(screen[(x + 3, 20)].fg, Color::LightRed, "error count");
        assert_eq!(screen[(x + 5, 20)].fg, LIGHT_BLUE, "separator");
        assert_eq!(screen[(x + 7, 20)].fg, Color::Yellow, "W");
        assert_eq!(screen[(x + 10, 20)].fg, Color::Yellow, "warning count");
    }
}
