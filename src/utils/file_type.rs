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

use std::path::Path;

pub(crate) fn display_name(language: &str) -> &str {
    match language {
        "text" => "Text",
        "rust" => "Rust",
        "javascript" => "JavaScript",
        "typescript" => "TypeScript",
        "python" => "Python",
        "go" => "Go",
        "java" => "Java",
        "c_sharp" => "C#",
        "c" => "C",
        "cpp" => "C++",
        "html" => "HTML",
        "css" => "CSS",
        "yaml" => "YAML",
        "json" => "JSON",
        "toml" => "TOML",
        "shell" => "Shell",
        "markdown" => "Markdown",
        _ => language,
    }
}

// Detect supported languages
pub(crate) fn detect(path: &Path, content: &str) -> &'static str {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    if matches!(
        filename,
        ".bashrc" | ".bash_profile" | ".profile" | ".zshrc"
    ) {
        return "shell";
    }

    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    if matches!(extension, "C" | "H") {
        return "cpp";
    }

    match extension.to_ascii_lowercase().as_str() {
        "rs" => "rust",
        "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "ts" | "mts" | "cts" => "typescript",
        "py" | "pyw" | "pyi" => "python",
        "go" => "go",
        "java" => "java",
        "cs" => "c_sharp",
        "c" | "h" => "c",
        "cc" | "cpp" | "cxx" | "hh" | "hpp" | "hxx" => "cpp",
        "html" | "htm" => "html",
        "css" => "css",
        "yaml" | "yml" => "yaml",
        "json" => "json",
        "toml" => "toml",
        "sh" | "bash" | "zsh" => "shell",
        "md" | "markdown" => "markdown",
        "txt" => "text",
        _ => detect_shebang(content).unwrap_or("text"),
    }
}

// A shebang is #! we have some example here: #!/bin/bash and #!/usr/bin/env python3.
fn detect_shebang(content: &str) -> Option<&'static str> {
    let first_line = content.lines().next()?;
    let mut words = first_line.strip_prefix("#!")?.split_whitespace();
    let executable = Path::new(words.next()?).file_name()?.to_str()?;
    let interpreter = if executable == "env" {
        words.find(|word| !word.starts_with('-') && !word.contains('='))?
    } else {
        executable
    };

    match interpreter {
        "sh" | "bash" | "dash" | "zsh" | "ksh" => Some("shell"),
        "node" | "nodejs" => Some("javascript"),
        name if name == "python"
            || name.strip_prefix("python").is_some_and(|suffix| {
                !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit() || c == '.')
            }) =>
        {
            Some("python")
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_code_editor::{editor::Editor, theme::vesper};

    #[test]
    fn recognized_types_have_working_highlighters() {
        for (name, language) in [
            ("main.rs", "rust"),
            ("app.jsx", "javascript"),
            ("app.ts", "typescript"),
            ("tool.py", "python"),
            ("main.go", "go"),
            ("Main.java", "java"),
            ("Main.cs", "c_sharp"),
            ("main.c", "c"),
            ("main.C", "cpp"),
            ("lib.hpp", "cpp"),
            ("index.html", "html"),
            ("style.css", "css"),
            ("build.yml", "yaml"),
            ("data.json", "json"),
            ("Cargo.toml", "toml"),
            ("script.sh", "shell"),
            (".bashrc", "shell"),
            ("README.MD", "markdown"),
        ] {
            assert_eq!(detect(Path::new(name), ""), language, "{name}");
            let editor = Editor::new(language, "", vesper()).unwrap();
            assert!(editor.code_ref().is_highlight(), "{name}");
        }
    }

    #[test]
    fn shebangs_and_plain_text_fallback() {
        for (content, language) in [
            ("#!/bin/bash", "shell"),
            ("#!/usr/bin/env -S python3 -u", "python"),
            ("#!/usr/bin/python3.12", "python"),
            ("#!/usr/bin/env node", "javascript"),
            ("#!/usr/bin/unknown", "text"),
            ("ordinary text", "text"),
        ] {
            assert_eq!(detect(Path::new("script"), content), language);
        }
        assert_eq!(detect(Path::new("note.txt"), "#!/bin/bash"), "text");
        assert_eq!(detect(Path::new("unknown.xyz"), "fn main() {}"), "text");
    }
}
