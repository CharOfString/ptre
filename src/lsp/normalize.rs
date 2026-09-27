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

use super::position::{Encoding, to_offset};
use lsp_types::{CompletionItem, CompletionResponse, CompletionTextEdit, InsertTextFormat};
use std::{iter::Peekable, ops::Range, str::Chars};

// A completion choice in editor terms, whichever source produced it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Candidate {
    pub(crate) label: String,
    pub(crate) detail: Option<String>,
    // Char range replaced by `text`, measured when the completion was requested.
    pub(crate) range: Range<usize>,
    pub(crate) text: String,
    sort_key: String,
    filter_key: String,
}

impl Candidate {
    // A candidate that simply replaces `range` with `word`.
    pub(crate) fn plain(word: &str, range: Range<usize>) -> Self {
        Self {
            label: word.to_owned(),
            detail: None,
            range,
            text: word.to_owned(),
            sort_key: word.to_owned(),
            filter_key: word.to_owned(),
        }
    }

    // Case-insensitive fuzzy match: `prefix` appears in order within the filter key.
    pub(crate) fn matches(&self, prefix: &str) -> bool {
        let mut key = self.filter_key.chars().flat_map(char::to_lowercase);
        prefix
            .chars()
            .flat_map(char::to_lowercase)
            .all(|wanted| key.any(|c| c == wanted))
    }
}

// Normalized answer to one completion request.
#[derive(Debug, Default)]
pub(crate) struct Completions {
    pub(crate) candidates: Vec<Candidate>,
    // The server did not send everything; typing further should ask again.
    pub(crate) incomplete: bool,
}

// Turn any server response shape into sorted candidates. `text` is the document the request
// was made against and `word` the range to replace when an item carries no edit of its own.
pub(crate) fn normalize(
    response: Option<CompletionResponse>,
    text: &str,
    word: Range<usize>,
    encoding: Encoding,
) -> Completions {
    let (items, incomplete) = match response {
        None => (Vec::new(), false),
        Some(CompletionResponse::Array(items)) => (items, false),
        Some(CompletionResponse::List(list)) => (list.items, list.is_incomplete),
    };
    let mut candidates: Vec<_> = items
        .into_iter()
        .map(|item| candidate(item, text, &word, encoding))
        .collect();
    candidates.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
    Completions {
        candidates,
        incomplete,
    }
}

fn candidate(
    item: CompletionItem,
    text: &str,
    word: &Range<usize>,
    encoding: Encoding,
) -> Candidate {
    // Prefer the explicit edit, then `insertText`, then the label itself.
    let convert = |range: lsp_types::Range| {
        let start = to_offset(text, range.start, encoding);
        start..to_offset(text, range.end, encoding).max(start)
    };
    let (range, new_text) = match item.text_edit {
        Some(CompletionTextEdit::Edit(edit)) => (convert(edit.range), edit.new_text),
        Some(CompletionTextEdit::InsertAndReplace(edit)) => (convert(edit.replace), edit.new_text),
        None => (
            word.clone(),
            item.insert_text.unwrap_or_else(|| item.label.clone()),
        ),
    };
    let new_text = if item.insert_text_format == Some(InsertTextFormat::SNIPPET) {
        strip_snippet(&new_text)
    } else {
        new_text
    };

    // clangd prefixes labels with ' ' or '•' (when it would also insert an #include).
    let label = item
        .label
        .trim_start_matches(|c: char| c == '•' || c.is_whitespace())
        .trim_end()
        .to_owned();
    Candidate {
        detail: item
            .detail
            .map(|detail| detail.trim().to_owned())
            .filter(|detail| !detail.is_empty()),
        range,
        text: new_text,
        sort_key: item.sort_text.unwrap_or_else(|| label.clone()),
        filter_key: item.filter_text.unwrap_or_else(|| label.clone()),
        label,
    }
}

// Reduce snippet syntax to its default text: `f(${1:x}, $2)$0` becomes `f(x, )`.
fn strip_snippet(snippet: &str) -> String {
    let mut out = String::new();
    let mut chars = snippet.chars().peekable();
    let mut depth = 0;
    while let Some(c) = chars.next() {
        match c {
            '\\' => out.extend(chars.next()),
            '}' if depth > 0 => depth -= 1,
            '$' if chars.peek() == Some(&'{') => {
                chars.next();
                skip_name(&mut chars);
                match chars.next() {
                    // Placeholder: keep its text, the matching '}' is dropped later.
                    Some(':') => depth += 1,
                    // Choice: keep the first option.
                    Some('|') => {
                        while let Some(c) = chars.next_if(|c| !matches!(c, ',' | '|')) {
                            out.push(c);
                        }
                        chars.by_ref().find(|&c| c == '}');
                    }
                    _ => {}
                }
            }
            '$' if chars.peek().is_some_and(|&c| is_name_char(c)) => skip_name(&mut chars),
            _ => out.push(c),
        }
    }
    out
}

fn skip_name(chars: &mut Peekable<Chars>) {
    while chars.next_if(|&c| is_name_char(c)).is_some() {}
}

fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn run(response: Value, text: &str, word: Range<usize>) -> Completions {
        let response = serde_json::from_value(response).unwrap();
        normalize(response, text, word, Encoding::Utf16)
    }

    #[test]
    fn snippets_keep_placeholder_defaults() {
        assert_eq!(
            strip_snippet("printf(${1:const char *fmt}, ${2:...})$0"),
            "printf(const char *fmt, ...)"
        );
        assert_eq!(
            strip_snippet(r"${1|one,two|} \$HOME $TM_FILENAME ${2:a${3:b}c}${4}"),
            "one $HOME  abc"
        );
        assert_eq!(strip_snippet("a $ b"), "a $ b");
    }

    #[test]
    fn clangd_style_list_uses_text_edits_and_sort_text() {
        // Shape of clangd 19 replies: a list, prefixed labels, filterText/sortText, text edits.
        let text = "int main(void) { 😀pri }";
        let edit = |new_text: &str| {
            json!({"newText": new_text, "range": {
                "start": {"line": 0, "character": 19}, "end": {"line": 0, "character": 22}}})
        };
        let completions = run(
            json!({"isIncomplete": true, "items": [
                {"label": " printf(const char *, ...)", "detail": "int", "sortText": "b",
                 "filterText": "printf", "insertTextFormat": 1, "textEdit": edit("printf")},
                {"label": "•print_it()", "sortText": "a", "filterText": "print_it",
                 "insertTextFormat": 2, "textEdit": edit("print_it(${1:x})")}
            ]}),
            text,
            18..21,
        );
        assert!(completions.incomplete);
        let [first, second] = completions.candidates.as_slice() else {
            panic!("expected two candidates");
        };
        assert_eq!(
            (first.label.as_str(), first.text.as_str()),
            ("print_it()", "print_it(x)")
        );
        assert_eq!(second.label, "printf(const char *, ...)");
        assert_eq!(second.detail.as_deref(), Some("int"));
        assert_eq!(second.range, 18..21);
        assert!(second.matches("prf") && second.matches("PRI") && !second.matches("fp"));
    }

    #[test]
    fn array_responses_fall_back_to_insert_text_label_and_word() {
        let completions = run(
            json!([
                {"label": "len", "insertText": "len()"},
                {"label": "iter", "detail": "  ", "textEdit": {"newText": "iter", "insert":
                    {"start": {"line": 1, "character": 2}, "end": {"line": 1, "character": 3}},
                    "replace":
                    {"start": {"line": 1, "character": 2}, "end": {"line": 1, "character": 5}}}}
            ]),
            "v.\nv.itx",
            5..6,
        );
        assert!(!completions.incomplete);
        let texts: Vec<_> = completions
            .candidates
            .iter()
            .map(|c| (c.text.as_str(), c.range.clone(), c.detail.clone()))
            .collect();
        assert_eq!(
            texts,
            [("iter", 5..8, None), ("len()", 5..6, None)],
            "sorted by label, replace range used"
        );
        assert!(run(Value::Null, "", 0..0).candidates.is_empty());
    }
}
