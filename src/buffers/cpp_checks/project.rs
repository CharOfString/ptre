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

//! Run the C/C++ checks while opening projects

use super::Tool;
use std::{ffi::OsString, path::Path, process::Command};

// Configuration files.
const CONFIG_FILES: [&str; 7] = [
    "CPPLINT.cfg",
    ".clang-tidy",
    ".clang-format",
    "_clang-format",
    "compile_commands.json",
    "build/compile_commands.json",
    "compile_flags.txt",
];

// The config files in root.
pub(crate) fn config_files(root: &Path) -> Vec<&'static str> {
    CONFIG_FILES
        .into_iter()
        .filter(|file| root.join(file).is_file())
        .collect()
}

// The command that checks file.
pub(super) fn command(tool: Tool, program: &str, file: &Path, project: Option<&Path>) -> Command {
    let mut command = Command::new(program);
    if let Some(root) = project.filter(|root| file.starts_with(root)) {
        command.current_dir(root);
        match tool {
            Tool::Cpplint => {
                let mut option = OsString::from("--repository=");
                option.push(root);
                command.arg(option);
            }

            Tool::Tidy => {
                let database = [root.to_path_buf(), root.join("build")]
                    .into_iter()
                    .find(|dir| dir.join("compile_commands.json").is_file());
                if let Some(dir) = database {
                    command.arg("-p").arg(dir);
                }
            }
        }
    }
    command.arg(file);
    command
}
