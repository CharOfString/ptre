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

use ratatui_code_editor::{
    code::{Code, Operation},
    editor::Editor,
    theme::vesper,
};
use std::{fs, io, path::PathBuf};

// State struct of an editor buffer.
pub(crate) struct Buffer {
    pub(crate) editor: Editor,
    pub(crate) path: Option<PathBuf>,
    saved_content: String,
    disk_content: Vec<u8>,
    pub(crate) obsolete: bool,
}

// Default editor configuration
impl Default for Buffer {
    fn default() -> Self {
        Self {
            editor: Editor::new("text", "", vesper())
                .expect("built-in editor configuration must be valid"),
            path: None,
            saved_content: String::new(),
            disk_content: Vec::new(),
            obsolete: false,
        }
    }
}

// Implementation of the editor buffer.
impl Buffer {
    pub(crate) fn open(&mut self, path: PathBuf) -> io::Result<()> {
        let content = fs::read_to_string(&path)?;
        let language = crate::utils::file_type::detect(&path, &content);
        let editor = Editor::new(language, &content, vesper())
            .map_err(|error| io::Error::other(error.to_string()))?;
        self.editor = editor;
        self.path = Some(path);
        self.saved_content = self.editor.get_content();
        self.disk_content = content.into_bytes();
        self.obsolete = false;
        Ok(())
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.editor.get_content() != self.saved_content
    }

    // Compare bytes for dirty buffer checks.
    pub(crate) fn check_disk(&mut self) {
        self.obsolete = self.path.as_ref().is_some_and(|path| {
            fs::read(path).map_or(true, |content| content != self.disk_content)
        });
    }

    pub(crate) fn save(&mut self) -> io::Result<()> {
        let path = self.path.clone().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Buffer has no 'file path'.")
        })?;
        self.save_as(path)
    }

    pub(crate) fn save_as(&mut self, path: PathBuf) -> io::Result<()> {
        let content = self.editor.get_content();

        let language = crate::utils::file_type::detect(&path, &content);
        let code = if language != self.editor.code_ref().lang() {
            Some(self.set_buffer_language(language)?)
        } else {
            None
        };

        // Only update the language, path and snapshots after a successful write.
        fs::write(&path, &content)?;
        if let Some(code) = code {
            *self.editor.code_mut() = code;
        }
        self.path = Some(path);
        self.disk_content = content.as_bytes().to_vec();
        self.saved_content = content;
        self.obsolete = false;
        Ok(())
    }

    // Set code language.
    fn set_buffer_language(&mut self, language: &str) -> io::Result<Code> {
        let old = self.editor.code_mut();
        old.commit();
        let mut past = Vec::new();
        while let Some(batch) = old.undo() {
            past.push(batch);
        }
        let initial = old.get_content();
        for _ in &past {
            old.redo();
        }
        let mut future = Vec::new();
        while let Some(batch) = old.redo() {
            future.push(batch);
        }
        for _ in &future {
            old.undo();
        }

        let mut code = Code::new(&initial, language, None)
            .map_err(|error| io::Error::other(error.to_string()))?;
        for batch in past.iter().rev().chain(&future) {
            code.tx();
            if let Some(state) = batch.state_before {
                code.set_state_before(state.offset, state.selection);
            }
            for edit in &batch.edits {
                match edit.operation {
                    Operation::Insert => code.insert(edit.start, &edit.text),
                    Operation::Remove => {
                        code.remove(edit.start, edit.start + edit.text.chars().count());
                    }
                }
            }
            if let Some(state) = batch.state_after {
                code.set_state_after(state.offset, state.selection);
            }
            code.commit();
        }
        for _ in &future {
            code.undo();
        }
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestFile(PathBuf);
    impl TestFile {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "ptre-watch-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::write(&path, "original").unwrap();
            Self(path)
        }
    }
    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn opening_files_switches_highlighting_and_failed_open_preserves_buffer() {
        let mut buffer = Buffer::default();
        assert!(!buffer.editor.code_ref().is_highlight());
        for (extension, content, language) in [
            ("rs", "fn main() {}", "rust"),
            ("py", "print(1)", "python"),
            ("txt", "plain text", "text"),
            ("", "#!/bin/bash\necho hello", "shell"),
        ] {
            let mut file = TestFile::new();
            let path = file.0.with_extension(extension);
            fs::rename(&file.0, &path).unwrap();
            file.0 = path;
            fs::write(&file.0, content).unwrap();
            buffer.open(file.0.clone()).unwrap();
            assert_eq!(buffer.editor.code_ref().lang(), language);
            assert_eq!(buffer.editor.code_ref().is_highlight(), language != "text");
            assert_eq!(buffer.editor.get_content(), content);
            assert!(!buffer.is_dirty());
            assert!(buffer.open(file.0.join("missing")).is_err());
            assert_eq!(buffer.editor.code_ref().lang(), language);
            assert_eq!(buffer.editor.get_content(), content);
            assert_eq!(buffer.path.as_ref(), Some(&file.0));
        }
    }

    #[test]
    fn saving_refreshes_language_and_preserves_cursor_and_history() {
        let mut file = TestFile::new();
        let path = file.0.with_extension("py");
        fs::rename(&file.0, &path).unwrap();
        file.0 = path;
        let mut buffer = Buffer::default();
        buffer.editor.set_content("print(1)");
        let code = buffer.editor.code_mut();
        code.tx();
        code.insert(8, "\n");
        code.commit();
        code.tx();
        code.insert(9, "# comment");
        code.commit();
        code.undo();
        buffer.editor.set_cursor(3);
        buffer.save_as(file.0.clone()).unwrap();
        assert_eq!(buffer.editor.code_ref().lang(), "python");
        assert!(buffer.editor.code_ref().is_highlight());
        assert_eq!(buffer.editor.get_cursor(), 3);
        assert!(!buffer.is_dirty());
        buffer.editor.code_mut().redo().unwrap();
        assert_eq!(buffer.editor.get_content(), "print(1)\n# comment");
        buffer.editor.code_mut().undo().unwrap();
        buffer.editor.code_mut().undo().unwrap();
        assert_eq!(buffer.editor.get_content(), "print(1)");
    }

    #[test]
    fn saving_redetects_shebang_and_failed_save_keeps_language() {
        let file = TestFile::new();
        let mut buffer = Buffer::default();
        buffer.open(file.0.clone()).unwrap();
        for (content, language) in [
            ("#!/bin/bash\necho hello", "shell"),
            ("#!/usr/bin/env python3\nprint(1)", "python"),
            ("plain text", "text"),
        ] {
            buffer.editor.set_content(content);
            buffer.save().unwrap();
            assert_eq!(buffer.editor.code_ref().lang(), language);
            assert_eq!(buffer.editor.code_ref().is_highlight(), language != "text");
            assert_eq!(buffer.editor.get_content(), content);
        }
        buffer.editor.set_content("#!/bin/bash");
        assert!(buffer.save_as(file.0.join("invalid.rs")).is_err());
        assert_eq!(buffer.editor.code_ref().lang(), "text");
        assert_eq!(buffer.path.as_ref(), Some(&file.0));
        assert!(buffer.is_dirty());
    }

    #[test]
    fn edits_and_disk_changes_are_independent_and_save_resets_both() {
        let file = TestFile::new();
        let mut buffer = Buffer::default();
        buffer.open(file.0.clone()).unwrap();
        buffer.check_disk();
        assert!(!buffer.is_dirty());
        assert!(!buffer.obsolete);
        buffer.editor.set_content("edited");
        assert!(buffer.is_dirty());
        buffer.check_disk();
        assert!(!buffer.obsolete);
        fs::write(&file.0, "external").unwrap();
        buffer.check_disk();
        assert!(buffer.obsolete);
        assert_eq!(buffer.editor.get_content(), "edited");
        buffer.save().unwrap();
        buffer.check_disk();
        assert!(!buffer.is_dirty());
        assert!(!buffer.obsolete);
        assert_eq!(fs::read_to_string(&file.0).unwrap(), "edited");
    }

    #[test]
    fn disk_changes_deletion_and_replacement_do_not_modify_buffer() {
        let file = TestFile::new();
        let mut buffer = Buffer::default();
        buffer.open(file.0.clone()).unwrap();
        fs::write(&file.0, "external").unwrap(); // Same length as original.
        buffer.check_disk();
        assert!(buffer.obsolete);
        assert!(!buffer.is_dirty());
        fs::remove_file(&file.0).unwrap();
        buffer.check_disk();
        assert!(buffer.obsolete);
        let replacement = TestFile::new();
        fs::rename(&replacement.0, &file.0).unwrap();
        buffer.check_disk();
        assert!(!buffer.obsolete); // Original bytes restored.
        assert_eq!(buffer.editor.get_content(), "original");
    }

    #[test]
    fn reverting_edits_and_failed_save_preserve_snapshots() {
        let file = TestFile::new();
        let mut buffer = Buffer::default();
        buffer.open(file.0.clone()).unwrap();
        buffer.editor.set_content("edited");
        assert!(buffer.is_dirty());
        assert!(buffer.save_as(file.0.join("invalid")).is_err());
        assert_eq!(buffer.path.as_ref(), Some(&file.0));
        assert!(buffer.is_dirty());
        buffer.editor.set_content("original");
        assert!(!buffer.is_dirty());
    }
}
