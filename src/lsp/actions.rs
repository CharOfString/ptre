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

/// Quick fixes from textDocument/codeAction.

use super::{
    position::{Encoding, to_offset, to_position},
    workspace,
};
use serde_json::{Value, json};
use std::ops::Range;

// One quick fix: a title and the edits it makes to the open document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Fix {
    pub(crate) title: String,
    // Char ranges in the text the fixes were asked for, with their new text. They never overlap.
    pub(crate) edits: Vec<(Range<usize>, String)>,
}

// The answer to one request, and the text its edits apply to.
#[derive(Debug, Default)]
pub(crate) struct Fixes {
    pub(crate) fixes: Vec<Fix>,
    pub(crate) text: String,
}

// Raw server diagnostics (as published) that touch `cursor`. The server matches the
// diagnostics of a request against its own, so they must be sent back unchanged.
pub(crate) fn diagnostics_at(
    raw: &[Value],
    text: &str,
    cursor: usize,
    encoding: Encoding,
) -> Vec<Value> {
    let offset = |position: &Value| {
        let position = serde_json::from_value(position.clone()).ok()?;
        Some(to_offset(text, position, encoding))
    };
    raw.iter()
        .filter(|diagnostic| {
            let range = &diagnostic["range"];
            let (Some(start), Some(end)) = (offset(&range["start"]), offset(&range["end"])) else {
                return false;
            };
            start <= cursor && cursor <= end
        })
        .cloned()
        .collect()
}

// Parameters of a request for quick fixes of `diagnostics` at `cursor`.
pub(crate) fn request(
    uri: &str,
    text: &str,
    cursor: usize,
    diagnostics: Vec<Value>,
    encoding: Encoding,
) -> Value {
    let position = to_position(text, cursor, encoding);
    json!({
        "textDocument": {"uri": uri},
        "range": {"start": position, "end": position},
        "context": {"diagnostics": diagnostics, "only": ["quickfix"], "triggerKind": 1},
    })
}

// Quick fixes of a response that only edit the document at `uri`, preferred ones first.
// Bare commands and fixes that touch other files are left out.
pub(crate) fn parse(result: &Value, uri: &str, text: &str, encoding: Encoding) -> Vec<Fix> {
    let actions = result.as_array().into_iter().flatten();
    let mut fixes: Vec<(bool, Fix)> = actions
        .filter(|action| {
            action["kind"]
                .as_str()
                .is_none_or(|k| k.starts_with("quickfix"))
        })
        .filter_map(|action| {
            let edits = document_edits(action.get("edit")?, uri)?;
            let edits = edits
                .into_iter()
                .map(|edit| {
                    let edit: lsp_types::TextEdit = serde_json::from_value(edit.clone()).ok()?;
                    let start = to_offset(text, edit.range.start, encoding);
                    let end = to_offset(text, edit.range.end, encoding).max(start);
                    Some((start..end, edit.new_text))
                })
                .collect::<Option<Vec<_>>>()?;
            let title = action["title"].as_str()?.to_owned();
            let preferred = action["isPreferred"].as_bool().unwrap_or(false);
            Some((preferred, Fix { title, edits }))
        })
        .collect();
    // A stable sort keeps the server's order among equals.
    fixes.sort_by_key(|(preferred, _)| !preferred);
    fixes.into_iter().map(|(_, fix)| fix).collect()
}

// The text edits of a workspace edit, if all of them are in `uri`.
fn document_edits<'a>(edit: &'a Value, uri: &str) -> Option<Vec<&'a Value>> {
    let mut edits = Vec::new();
    if let Some(changes) = edit["documentChanges"].as_array() {
        for change in changes {
            // Create, rename and delete operations have a `kind`; text edits do not.
            let target = change["textDocument"]["uri"].as_str()?;
            if change.get("kind").is_some() || !workspace::same_uri(target, uri) {
                return None;
            }
            edits.extend(change["edits"].as_array()?);
        }
    } else {
        for (target, list) in edit["changes"].as_object()? {
            if !workspace::same_uri(target, uri) {
                return None;
            }
            edits.extend(list.as_array()?);
        }
    }
    (!edits.is_empty()).then_some(edits)
}

#[cfg(test)]
mod tests {
    use super::*;

    const URI: &str = "file:///tmp/main.c";

    fn range(line: u32, start: u32, end: u32) -> Value {
        json!({"start": {"line": line, "character": start},
               "end": {"line": line, "character": end}})
    }

    #[test]
    fn requests_carry_the_raw_diagnostics_at_the_cursor() {
        let text = "int x;\nint y = valu;\n";
        let raw = [
            json!({"range": range(1, 8, 12), "message": "undeclared 'valu'", "code": "x"}),
            json!({"range": range(0, 4, 5), "message": "unused 'x'"}),
        ];
        let at = diagnostics_at(&raw, text, 19, Encoding::Utf16);
        assert_eq!(at, [raw[0].clone()]);
        assert!(diagnostics_at(&raw, text, 14, Encoding::Utf16).is_empty());
        let params = request(URI, text, 19, at, Encoding::Utf16);
        assert_eq!(params["range"], range(1, 12, 12));
        assert_eq!(params["context"]["only"], json!(["quickfix"]));
        assert_eq!(params["context"]["diagnostics"][0]["code"], "x");
    }

    #[test]
    fn only_edits_of_this_document_become_fixes() {
        // Shape of clangd 19 output, which uses `changes`.
        let text = "int value;\nint y = valu;\n";
        let edit =
            |uri: &str| json!({"changes": {uri: [{"range": range(1, 8, 12), "newText": "value"}]}});
        let result = json!([
            {"title": "change 'valu' to 'value'", "kind": "quickfix", "edit": edit(URI)},
            {"title": "preferred", "kind": "quickfix", "isPreferred": true, "edit": {
                "documentChanges": [{"textDocument": {"uri": URI, "version": 1},
                                     "edits": [{"range": range(1, 12, 12), "newText": ";"}]}]}},
            {"title": "other file", "kind": "quickfix", "edit": edit("file:///tmp/b.c")},
            {"title": "command only", "command": "clangd.applyTweak"},
            {"title": "refactor", "kind": "refactor.extract", "edit": edit(URI)},
            {"title": "create file", "edit": {"documentChanges": [
                {"kind": "create", "uri": "file:///tmp/new.c"}]}},
        ]);
        let fixes = parse(&result, URI, text, Encoding::Utf16);
        assert_eq!(
            fixes,
            [
                Fix {
                    title: "preferred".into(),
                    edits: vec![(23..23, ";".into())],
                },
                Fix {
                    title: "change 'valu' to 'value'".into(),
                    edits: vec![(19..23, "value".into())],
                },
            ]
        );
        assert!(parse(&Value::Null, URI, text, Encoding::Utf16).is_empty());
    }
}
