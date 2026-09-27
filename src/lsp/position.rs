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

use lsp_types::Position;

// How a server counts the `character` column of a position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Encoding {
    Utf8,
    Utf16,
    Utf32,
}

impl Encoding {
    // Encodings we offer in `initialize`, most preferred first. UTF-32 equals editor offsets.
    pub(crate) const OFFERED: [&str; 2] = ["utf-32", "utf-16"];

    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "utf-8" => Some(Self::Utf8),
            "utf-16" => Some(Self::Utf16),
            "utf-32" => Some(Self::Utf32),
            _ => None,
        }
    }

    fn width(self, c: char) -> u32 {
        match self {
            Self::Utf8 => c.len_utf8() as u32,
            Self::Utf16 => c.len_utf16() as u32,
            Self::Utf32 => 1,
        }
    }
}

// Convert an editor char offset into an LSP position.
pub(crate) fn to_position(text: &str, offset: usize, encoding: Encoding) -> Position {
    let (mut line, mut character) = (0, 0);
    for c in text.chars().take(offset) {
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            character += encoding.width(c);
        }
    }
    Position::new(line, character)
}

// Convert an LSP position into an editor char offset, clamping to the line and the text.
pub(crate) fn to_offset(text: &str, position: Position, encoding: Encoding) -> usize {
    let (mut line, mut character) = (0, 0);
    for (offset, c) in text.chars().enumerate() {
        if line == position.line {
            if character >= position.character || c == '\n' {
                return offset;
            }
            character += encoding.width(c);
        } else if c == '\n' {
            line += 1;
        }
    }
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_count_columns_in_the_negotiated_encoding() {
        // 'é' is 2 UTF-8 bytes, '😀' is 2 UTF-16 units and 4 UTF-8 bytes.
        let text = "ab\né😀x\nlast";
        let x = text.chars().position(|c| c == 'x').unwrap();
        for (encoding, column) in [
            (Encoding::Utf32, 2),
            (Encoding::Utf16, 3),
            (Encoding::Utf8, 6),
        ] {
            let position = to_position(text, x, encoding);
            assert_eq!(position, Position::new(1, column), "{encoding:?}");
            assert_eq!(to_offset(text, position, encoding), x, "{encoding:?}");
        }
    }

    #[test]
    fn out_of_range_positions_are_clamped() {
        let text = "ab\ncd";
        assert_eq!(to_offset(text, Position::new(0, 99), Encoding::Utf16), 2);
        assert_eq!(to_offset(text, Position::new(9, 0), Encoding::Utf16), 5);
        assert_eq!(to_position(text, 99, Encoding::Utf16), Position::new(1, 2));
        assert_eq!(Encoding::from_name("utf-7"), None);
    }
}
