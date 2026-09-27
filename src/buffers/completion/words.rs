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

use crate::lsp::Candidate;
use std::{collections::HashSet, ops::Range};

pub(super) fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

// Start of the identifier that ends at `cursor`.
pub(super) fn word_start(text: &str, cursor: usize) -> usize {
    let before: Vec<char> = text.chars().take(cursor).collect();
    cursor
        - before
            .iter()
            .rev()
            .take_while(|&&c| is_word_char(c))
            .count()
}

// Distinct identifiers from the buffer, nearest to the cursor first. The word being typed and
// plain numbers are left out.
pub(super) fn buffer_words(text: &str, word: Range<usize>) -> Vec<Candidate> {
    let mut found = Vec::new();
    let mut current = String::new();
    let mut start = 0;
    for (offset, c) in text.chars().chain([' ']).enumerate() {
        if is_word_char(c) {
            if current.is_empty() {
                start = offset;
            }
            current.push(c);
        } else if !current.is_empty() {
            found.push((start, std::mem::take(&mut current)));
        }
    }
    found.retain(|(start, found)| {
        *start != word.start && found.chars().count() > 1 && !found.starts_with(char::is_numeric)
    });
    found.sort_by_key(|(start, _)| start.abs_diff(word.start));

    let mut seen = HashSet::new();
    found
        .into_iter()
        .filter(|(_, found)| seen.insert(found.clone()))
        .map(|(_, found)| Candidate::plain(&found, word.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_skip_the_current_word_numbers_and_duplicates() {
        let text = "alpha 42 x beta alpha\nal";
        assert_eq!(word_start(text, 24), 22);
        assert_eq!(word_start(text, 23), 22);
        assert_eq!(word_start(text, 5), 0);
        let words: Vec<_> = buffer_words(text, 22..24)
            .into_iter()
            .map(|candidate| (candidate.text, candidate.range))
            .collect();
        assert_eq!(
            words,
            [("alpha".into(), 22..24), ("beta".into(), 22..24)],
            "nearest first"
        );
    }
}
