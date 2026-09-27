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

use std::{
    fmt::Write,
    io,
    path::{Path, PathBuf},
};

// Files that mark the top of a project, checked from the file's directory upwards.
const ROOT_MARKERS: [&str; 5] = [
    "compile_commands.json",
    "compile_flags.txt",
    "CMakeLists.txt",
    "Cargo.toml",
    ".git",
];

// The workspace root for `file`: the nearest ancestor holding a marker, else its directory.
pub(crate) fn find_root(file: &Path) -> io::Result<PathBuf> {
    let file = std::path::absolute(file)?;
    let directory = file.parent().unwrap_or(Path::new("/"));
    let root = directory
        .ancestors()
        .find(|dir| ROOT_MARKERS.iter().any(|marker| dir.join(marker).exists()))
        .unwrap_or(directory);
    Ok(root.to_path_buf())
}

// A `file://` URI with everything outside the unreserved set percent-encoded.
pub(crate) fn file_uri(path: &Path) -> io::Result<String> {
    let path = std::path::absolute(path)?;
    let mut uri = String::from("file://");
    for &byte in path.as_os_str().as_encoded_bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte) {
            uri.push(byte as char);
        } else {
            let _ = write!(uri, "%{byte:02X}");
        }
    }
    Ok(uri)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots_and_uris_are_absolute() {
        let root = find_root(Path::new("src/lsp/workspace.rs")).unwrap();
        assert_eq!(root, std::env::current_dir().unwrap());
        assert_eq!(
            file_uri(Path::new("/tmp/a b/ü.c")).unwrap(),
            "file:///tmp/a%20b/%C3%BC.c"
        );
    }
}
