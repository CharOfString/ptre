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
}

// One color per severity, for underlines, dots, messages and popup labels.
fn color(severity: Severity) -> Color {
    match severity {
        Severity::Error => Color::LightRed,
        Severity::Warning => Color::Yellow,
        Severity::Information => LIGHT_BLUE,
        Severity::Hint => Color::Gray,
    }
}
