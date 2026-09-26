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

// State struct of an editor buffer.
#[derive(Default)]
pub(crate) struct Buffer {
    pub(crate) text: String,
    pub(crate) path: Option<PathBuf>,
}

// Implementation of the editor buffer.
impl Buffer {
    pub(crate) fn save(&self) -> io::Result<()> {
        let path = self.path.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Buffer has no 'file path'.")
        })?;
        fs::write(path, &self.text)
    }

    pub(crate) fn save_as(&mut self, path: PathBuf) -> io::Result<()> {
        // Only update buffer path while write succeed
        fs::write(&path, &self.text)?;
        self.path = Some(path);
        Ok(())
    }
}
