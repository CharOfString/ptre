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
mod lsp;
mod utils;

use app::App;
use crossterm::{
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute,
};
use std::{ffi::OsString, io, path::PathBuf};

// Main app of Pointer.
fn main() -> io::Result<()> {
    let mut app = App::default();
    app.load_cpp_preferences();

    // Open the passed in file, or the passed in folder as a project, if any.
    if let Some(path) = path_argument(std::env::args_os().skip(1))? {
        if path.is_dir() {
            app.open_project(path.canonicalize()?);
        } else {
            app.buffer.open(path)?;
        }
    }

    // Language server plugins; a broken plugin file is reported but does not stop ptre.
    let (plugins, errors) = lsp::load_plugins();
    app.completion.set_plugins(plugins);
    if let Some(error) = errors.first() {
        app.status_bar_text = format!("Plugin error: {error}");
    }

    ratatui::run(|terminal| {
        // Ask the terminal emulator to send pasted text in one piece, not as single keys.
        execute!(io::stdout(), EnableBracketedPaste)?;
        let result = app.run(terminal);
        execute!(io::stdout(), DisableBracketedPaste)?;
        result
    })
}

// Validate the path arguments
fn path_argument(mut args: impl Iterator<Item = OsString>) -> io::Result<Option<PathBuf>> {
    // Read first argument
    let Some(argument) = args.next() else {
        return Ok(None);
    };

    // We only allow to open one file or folder for each instance.
    if args.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Only one path is allowed.",
        ));
    }

    // Check if that is an actual file or folder.
    let path = PathBuf::from(argument);
    let metadata = std::fs::metadata(&path)?;
    if !metadata.is_file() && !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Only a regular file or a folder is allowed.",
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
            let path = path_argument([name.into_os_string()].into_iter())
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
    fn folders_open_as_projects() {
        let src = std::env::current_dir().unwrap().join("src");
        for name in ["src", "src/", "./src"] {
            let path = path_argument([OsString::from(name)].into_iter())
                .unwrap()
                .unwrap();
            let mut app = App::default();
            app.open_project(path.canonicalize().unwrap());
            assert_eq!(app.sidebar.project.as_ref(), Some(&src), "{name}");
            assert!(app.sidebar.visible && app.sidebar.focused, "{name}");
            assert_eq!(app.buffer.path, None, "{name}");
        }
    }

    #[test]
    fn rejects_missing_paths_devices_and_extra_arguments() {
        for name in ["missing-file", "/dev/null"] {
            assert!(
                path_argument([OsString::from(name)].into_iter()).is_err(),
                "{name}"
            );
        }
        assert!(
            path_argument([OsString::from("Cargo.toml"), OsString::from("README.md")].into_iter())
                .is_err()
        );
        assert!(path_argument(std::iter::empty()).unwrap().is_none());
    }
}
