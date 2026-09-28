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

/// Errors and warnings the server reports with textDocument/publishDiagnostics.

use super::position::{Encoding, to_offset};
use lsp_types::DiagnosticSeverity;
use serde_json::Value;
use std::ops::Range;

// Most severe first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

impl Severity {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Information => "info",
            Self::Hint => "hint",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Diagnostic {
    // Char range in the editor text.
    pub(crate) range: Range<usize>,
    pub(crate) severity: Severity,
    pub(crate) message: String,
    // The tool behind it, such as "clang" or "clang-tidy".
    pub(crate) source: Option<String>,
}

impl Diagnostic {
    // Whether the char at offset is marked.
    pub(crate) fn marks(&self, offset: usize) -> bool {
        offset >= self.range.start && offset < self.range.end.max(self.range.start + 1)
    }
}

// Read the diagnostics of a publish notification, sorted by position.
pub(crate) fn parse(params: &Value, text: &str, encoding: Encoding) -> Vec<Diagnostic> {
    let items: Vec<lsp_types::Diagnostic> =
        serde_json::from_value(params["diagnostics"].clone()).unwrap_or_default();
    let mut diagnostics: Vec<_> = items
        .into_iter()
        .map(|item| {
            let start = to_offset(text, item.range.start, encoding);
            let end = to_offset(text, item.range.end, encoding).max(start);
            Diagnostic {
                range: start..end,
                // The protocol leaves a missing severity to the client; treat it as an error.
                severity: match item.severity {
                    Some(DiagnosticSeverity::WARNING) => Severity::Warning,
                    Some(DiagnosticSeverity::INFORMATION) => Severity::Information,
                    Some(DiagnosticSeverity::HINT) => Severity::Hint,
                    _ => Severity::Error,
                },
                message: item.message,
                source: item.source,
            }
        })
        .collect();
    diagnostics.sort_by_key(|diagnostic| (diagnostic.range.start, diagnostic.severity));
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn diagnostics_are_converted_and_sorted() {
        // Shape of clangd 19 output; the columns count UTF-16 units past the emoji.
        let text = "int a;\n😀 x = 1; y;\n";
        let range = |line: u32, start: u32, end: u32| {
            json!({"start": {"line": line, "character": start},
                   "end": {"line": line, "character": end}})
        };
        let params = json!({"uri": "file:///tmp/main.c", "version": 3, "diagnostics": [
            {"range": range(1, 11, 12), "severity": 2, "message": "unused", "source": "clang"},
            {"range": range(1, 3, 4), "message": "unknown type name 'x'"},
            {"range": range(0, 0, 0), "severity": 4, "message": "hint"},
        ]});
        let diagnostics = parse(&params, text, Encoding::Utf16);
        let summary: Vec<_> = diagnostics
            .iter()
            .map(|d| (d.range.clone(), d.severity, d.message.as_str()))
            .collect();
        assert_eq!(
            summary,
            [
                (0..0, Severity::Hint, "hint"),
                (9..10, Severity::Error, "unknown type name 'x'"),
                (17..18, Severity::Warning, "unused"),
            ]
        );
        assert_eq!(diagnostics[2].source.as_deref(), Some("clang"));
        assert!(diagnostics[0].marks(0) && !diagnostics[0].marks(1));
        assert!(diagnostics[1].marks(9) && !diagnostics[1].marks(10));
        assert!(parse(&json!({"diagnostics": null}), text, Encoding::Utf16).is_empty());
    }
}
