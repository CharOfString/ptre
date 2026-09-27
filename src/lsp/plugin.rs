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

// Language server plugins. A plugin is one `.conf` file of `key = value` lines naming the
// command to run; the file name is the ptre language it serves, so `python.conf` serves Python.

use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Plugin {
    language: String,
    // Program followed by its arguments; never empty.
    pub(crate) command: Vec<String>,
    enabled: bool,
}

impl Plugin {
    pub(crate) fn serves(&self, language: &str) -> bool {
        self.enabled && self.language == language
    }
}

// Plugins compiled into ptre. A user plugin for the same language replaces one of these.
const BUILT_IN: [(&str, &str); 3] = [
    ("c", include_str!("../../plugins/lsp/c.conf")),
    ("cpp", include_str!("../../plugins/lsp/cpp.conf")),
    ("rust", include_str!("../../plugins/lsp/rust.conf")),
];

// Load user plugins and the built-in ones they do not replace. Files that fail to load are
// skipped and reported as `file.conf: problem` (the directory is always the same, and the
// status bar is short).
pub(crate) fn load() -> (Vec<Plugin>, Vec<String>) {
    load_from(user_dir().as_deref())
}

// `$XDG_CONFIG_HOME/ptre/plugins/lsp`, falling back to `~/.config/ptre/plugins/lsp`.
fn user_dir() -> Option<PathBuf> {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(config.join("ptre").join("plugins").join("lsp"))
}

fn load_from(dir: Option<&Path>) -> (Vec<Plugin>, Vec<String>) {
    let mut files: Vec<PathBuf> = dir
        .and_then(|dir| fs::read_dir(dir).ok())
        .into_iter()
        .flatten()
        .filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "conf")
        })
        .collect();
    files.sort();

    let mut plugins = Vec::new();
    let mut errors = Vec::new();
    for path in files {
        let language = path.file_stem().unwrap_or_default().to_string_lossy();
        let plugin = fs::read_to_string(&path)
            .map_err(|error| error.to_string())
            .and_then(|text| parse(&language, &text));
        match plugin {
            Ok(plugin) => plugins.push(plugin),
            Err(error) => errors.push(format!("{language}.conf: {error}")),
        }
    }
    for (language, text) in BUILT_IN {
        if !plugins.iter().any(|plugin| plugin.language == language) {
            plugins.push(parse(language, text).expect("built-in plugins are valid"));
        }
    }
    (plugins, errors)
}

// Parse the plugin file for `language`. Blank lines and lines starting with '#' are ignored.
pub(crate) fn parse(language: &str, text: &str) -> Result<Plugin, String> {
    if language_id(language).is_none() {
        return Err(format!(
            "`{language}` is not a ptre language; name the file after one, such as python.conf"
        ));
    }
    let (mut command, mut enabled) = (None, None);
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = |message: String| format!("line {}: {message}", index + 1);
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| at("expected `key = value`".into()))?;
        let (key, value) = (key.trim(), value.trim());
        match key {
            "command" if command.is_none() => command = Some(split_command(value).map_err(at)?),
            "enabled" if enabled.is_none() => {
                enabled = Some(match value {
                    "true" => true,
                    "false" => false,
                    _ => return Err(at("`enabled` must be true or false".into())),
                });
            }
            "command" | "enabled" => return Err(at(format!("`{key}` is set twice"))),
            _ => return Err(at(format!("unknown key `{key}`"))),
        }
    }
    Ok(Plugin {
        language: language.to_owned(),
        command: command.ok_or("missing `command`")?,
        enabled: enabled.unwrap_or(true),
    })
}

// Split a command line on whitespace; single or double quotes keep spaces in one argument.
fn split_command(line: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut word: Option<String> = None;
    let mut quote = None;
    for c in line.chars() {
        match quote {
            Some(open) if c == open => quote = None,
            Some(_) => word.get_or_insert_default().push(c),
            None if c == '"' || c == '\'' => {
                quote = Some(c);
                word.get_or_insert_default();
            }
            None if c.is_whitespace() => words.extend(word.take()),
            None => word.get_or_insert_default().push(c),
        }
    }
    if quote.is_some() {
        return Err("unclosed quote in `command`".into());
    }
    words.extend(word);
    if words.is_empty() {
        return Err("`command` is empty".into());
    }
    Ok(words)
}

// The LSP `languageId` for a ptre language name (see `utils::file_type`).
pub(crate) fn language_id(language: &str) -> Option<&'static str> {
    Some(match language {
        "text" => "plaintext",
        "rust" => "rust",
        "javascript" => "javascript",
        "typescript" => "typescript",
        "python" => "python",
        "go" => "go",
        "java" => "java",
        "c_sharp" => "csharp",
        "c" => "c",
        "cpp" => "cpp",
        "html" => "html",
        "css" => "css",
        "yaml" => "yaml",
        "json" => "json",
        "toml" => "toml",
        "shell" => "shellscript",
        "markdown" => "markdown",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_files_parse_with_comments_quotes_and_defaults() {
        let plugin = parse(
            "python",
            "# Python\n\n command = pyright-langserver --stdio 'a b'\"\"\n",
        )
        .unwrap();
        assert_eq!(plugin.command, ["pyright-langserver", "--stdio", "a b"]);
        assert!(plugin.serves("python") && !plugin.serves("rust"));
        let disabled = parse("c", "enabled = false\ncommand = x").unwrap();
        assert!(!disabled.serves("c"));
    }

    #[test]
    fn plugin_errors_name_the_line() {
        for (text, error) in [
            ("", "missing `command`"),
            ("command x", "line 1: expected `key = value`"),
            ("command = x\ncommand = y", "line 2: `command` is set twice"),
            ("\nlanguages = c", "line 2: unknown key `languages`"),
            ("command = 'x", "line 1: unclosed quote in `command`"),
            ("command =  ", "line 1: `command` is empty"),
            ("enabled = yes", "line 1: `enabled` must be true or false"),
        ] {
            assert_eq!(parse("c", text), Err(error.into()), "{text}");
        }
        let error = parse("Python", "command = x").unwrap_err();
        assert!(
            error.starts_with("`Python` is not a ptre language"),
            "{error}"
        );
    }

    #[test]
    fn user_plugins_replace_built_ins_of_the_same_language() {
        let dir = std::env::temp_dir().join(format!("ptre-plugins-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("c.conf"), "enabled = false\ncommand = clangd").unwrap();
        fs::write(dir.join("go.conf"), "command = gopls").unwrap();
        fs::write(dir.join("golang.conf"), "command = gopls").unwrap();
        fs::write(dir.join("notes.txt"), "not a plugin").unwrap();
        let (plugins, errors) = load_from(Some(&dir));
        fs::remove_dir_all(&dir).unwrap();

        let find = |language| plugins.iter().find(|plugin| plugin.serves(language));
        assert_eq!(find("go").unwrap().command, ["gopls"]);
        assert!(find("c").is_none(), "user file disabled the built-in");
        assert_eq!(find("cpp").unwrap().command, ["clangd"]);
        assert_eq!(plugins.len(), 4);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("golang.conf: `golang` is not a ptre language"));
    }

    #[test]
    fn built_ins_load_without_a_user_directory() {
        let (plugins, errors) = load_from(None);
        assert!(errors.is_empty());
        let find = |language| plugins.iter().find(|plugin| plugin.serves(language));
        assert_eq!(find("c").unwrap().command, ["clangd"]);
        assert_eq!(find("cpp").unwrap().command, ["clangd"]);
        assert_eq!(find("rust").unwrap().command, ["rust-analyzer"]);
        assert!(find("python").is_none());
    }

    #[test]
    fn language_ids_cover_ptre_language_names() {
        use crate::utils::file_type::display_name;
        for language in [
            "text",
            "rust",
            "javascript",
            "typescript",
            "python",
            "go",
            "java",
            "c_sharp",
            "c",
            "cpp",
            "html",
            "css",
            "yaml",
            "json",
            "toml",
            "shell",
            "markdown",
        ] {
            assert!(language_id(language).is_some(), "{language}");
            assert_ne!(display_name(language), language, "ptre knows {language}");
        }
    }
}
