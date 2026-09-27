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

mod app;
mod buffers;

use app::App;
use std::{ffi::OsString, io, path::PathBuf};

// Main app of Pointer.
fn main() -> io::Result<()> {
    // Attempt to open the passed in file, if any.
    let mut app = App::default();
    if let Some(path) = file_argument(std::env::args_os().skip(1))? {
        app.buffer.open(path)?;
    }

    ratatui::run(|terminal| app.run(terminal))
}

// Validate the file arguments
fn file_argument(mut args: impl Iterator<Item = OsString>) -> io::Result<Option<PathBuf>> {
    // Read first argument
    let Some(argument) = args.next() else {
        return Ok(None);
    };

    // We only allow to open one file for each instance.
    if args.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Only one file is allowed.",
        ));
    }

    // Check if that is an actual file.
    // TODO: To allow directories ONCE AFTER we implemented workspace feature.
    let path = PathBuf::from(argument);
    if !std::fs::metadata(&path)?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Only regular file (NOT a directory) is allowed.",
        ));
    }
    Ok(Some(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_relative_and_absolute_file_paths_and_opens_them() {
        let absolute = std::env::current_dir().unwrap().join("README.md");
        for name in ["Cargo.toml".into(), "src/main.rs".into(), absolute] {
            let path = file_argument([name.into_os_string()].into_iter())
                .unwrap()
                .unwrap();
            let mut app = App::default();
            app.buffer.open(path.clone()).unwrap();
            assert_eq!(app.buffer.path, Some(path.clone()));
            assert_eq!(
                app.buffer.editor.get_content(),
                std::fs::read_to_string(path).unwrap()
            );
        }
    }

    #[test]
    fn rejects_directories_missing_files_and_extra_arguments() {
        for name in ["src", ".", "missing-file"] {
            assert!(
                file_argument([OsString::from(name)].into_iter()).is_err(),
                "{name}"
            );
        }
        assert!(
            file_argument([OsString::from("Cargo.toml"), OsString::from("README.md")].into_iter())
                .is_err()
        );
        assert!(file_argument(std::iter::empty()).unwrap().is_none());
    }
}
