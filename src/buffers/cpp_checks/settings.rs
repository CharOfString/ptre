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

use std::{fs, io, path::PathBuf};

#[derive(Clone, Default)]
pub(crate) struct Settings {
    pub(crate) tidy: bool,
    pub(crate) cpplint: bool,
    pub(crate) cpplint_path: String,
}

pub(super) fn settings_path() -> Option<PathBuf> {
    let root = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(root.join("ptre/cpp-checks.json"))
}

impl Settings {
    pub(super) fn load(path: &std::path::Path) -> io::Result<Self> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };

        let value: serde_json::Value = serde_json::from_str(&text)?;
        Ok(Self {
            tidy: value["clang_tidy"].as_bool().unwrap_or(false),
            cpplint: value["cpplint"].as_bool().unwrap_or(false),
            cpplint_path: value["cpplint_path"].as_str().unwrap_or_default().into(),
        })
    }

    pub(super) fn save(&self, path: &std::path::Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let value = serde_json::json!({
            "clang_tidy": self.tidy,
            "cpplint": self.cpplint,
            "cpplint_path": self.cpplint_path,
        });
        // Replace the complete preferences file, never a partly written JSON file.
        let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
        fs::write(&temporary, serde_json::to_vec_pretty(&value)?)?;
        fs::rename(temporary, path)
    }
}
