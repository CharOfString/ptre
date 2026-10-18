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

use crate::buffers::editor::Buffer;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, layout::Rect};
use ratatui_code_editor::actions::{Delete, InsertText, Redo, Undo};
use std::{
    io,
    path::PathBuf,
    time::{Duration, Instant},
};

// The state struct of editor
#[derive(Default)]
pub(crate) struct App {
    pub(crate) buffer: Buffer,
    pub(crate) terminal: crate::buffers::terminal::Terminal,
    pub(crate) menu: crate::buffers::menu::Menu,
    pub(crate) editor_area: Rect,
    exit_flag: bool,
    pub(crate) ctrl_x_wait_flag: bool,
    pub(crate) ctrl_c_wait_flag: bool,
    // Set after C-c t, which starts terminal and tool commands.
    pub(crate) ctrl_c_t_wait_flag: bool,
    kill_ring: Option<String>,
    pub(crate) overwrite_confirm: bool,
    pub(crate) save_path_input: Option<String>,
    pub(crate) status_bar_text: String,
    pub(crate) buffer_command: String,
    pub(crate) cpp_checks: crate::buffers::cpp_checks::Checks,
    pub(crate) completion: crate::buffers::completion::Completion,
    pub(crate) quick_fix: crate::buffers::fixes::QuickFix,
    pub(crate) search: crate::buffers::search::Search,
    pub(crate) goto_line: crate::buffers::goto_line::GotoLine,
    pub(crate) paste_scroll: Option<crate::buffers::paste::Scroll>,
    pub(crate) sidebar: crate::buffers::sidebar::Sidebar,
    pub(crate) tabs: crate::buffers::tabs::Tabs,
    // Draw icons from a Nerd Font (WINDOW menu).
    pub(crate) nerd_font: bool,
}

// Implementation of the app struct
impl App {
    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // Watch the file each 500ms.
        let interval = Duration::from_millis(500);
        let mut last_disk_check = Instant::now() - interval;

        // The main loop: While the exit flag is false, keep ratatui running
        while !self.exit_flag {
            // Timer for check file state.
            if last_disk_check.elapsed() >= interval {
                self.buffer.check_disk();
                last_disk_check = Instant::now();
            }

            // Talk to the language server, then draw a new frame.
            self.sync_lsp();
            self.terminal.poll();
            if !self.ctrl_x_wait_flag && !self.ctrl_c_wait_flag && !self.ctrl_c_t_wait_flag {
                self.poll_cpp_check();
            }

            terminal.draw(|frame| self.draw(frame))?;

            // Wait for event; ignore key release to avoid handling a keystroke twice.
            // Wake up sooner while a completion answer is expected or the view scrolls.
            let mut timeout = interval.saturating_sub(last_disk_check.elapsed());
            if self.terminal.visible
                || self.completion.is_waiting()
                || self.quick_fix.is_waiting()
                || self.paste_scroll.is_some()
            {
                timeout = timeout.min(Duration::from_millis(20));
            }
            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) if key.kind != KeyEventKind::Release => self.handle_key(key),
                    // Pasted text comes in one piece and becomes one edit.
                    Event::Paste(text) => self.handle_paste(text),
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) {
        if self.handle_sidebar_key(key) || self.handle_terminal_key(key) {
            return;
        }
        // Print key in buffer command section in bottom tool bar.
        let prefix = match (self.ctrl_x_wait_flag, self.ctrl_c_wait_flag) {
            (true, _) => "C-x ",
            (_, true) => "C-c ",
            _ if self.goto_line.after_prefix() => "M-g ",
            _ => "",
        };
        self.buffer_command = crate::utils::keys::shortcut_label(key, !prefix.is_empty())
            .map_or_else(String::new, |chord| format!("{prefix}{chord}"));

        if self.handle_cpp_check_key(key) || self.handle_tab_key(key) {
            return;
        }

        // Conformation modal "dialog" for overwriting document.
        if self.overwrite_confirm {
            match key.code {
                // Press Y or ENTER for conforming.
                KeyCode::Char('y' | 'Y') | KeyCode::Enter
                    if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
                {
                    self.overwrite_confirm = false;
                    match self.buffer.save() {
                        Ok(()) => self.status_bar_text = "Saved".into(),
                        Err(err) => self.status_bar_text = format!("Save failed: {err}"),
                    }
                }

                // Press N or ESC to cancel.
                KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                    self.overwrite_confirm = false;
                    self.status_bar_text = "Save cancelled".into();
                }

                // Ignoring all other inputs.
                _ => {}
            }
            return;
        }

        // While entering a filename, keystrokes must not change the buffer.
        if let Some(input) = &mut self.save_path_input {
            match key.code {
                KeyCode::Esc => {
                    self.save_path_input = None;
                    self.status_bar_text = "Save cancelled".into();
                }
                KeyCode::Enter => {
                    if input.trim().is_empty() {
                        self.status_bar_text = "Enter a file path".into();
                        return;
                    }

                    let path = PathBuf::from(input.as_str());
                    match self.buffer.save_as(path) {
                        Ok(()) => {
                            self.save_path_input = None;
                            self.status_bar_text = "Saved".into();
                        }
                        Err(err) => self.status_bar_text = format!("Save failed: {err}"),
                    }
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(c)
                    if !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    input.push(c);
                }
                _ => {}
            }
            return;
        }

        // Search and replace see keys before completion.
        if !self.ctrl_x_wait_flag
            && self.menu.active.is_none()
            && (self.handle_search_key(key)
                || self.handle_goto_line_key(key)
                || self.handle_completion_key(key))
        {
            return;
        }

        if self.handle_menu_key(key) {
            self.ctrl_x_wait_flag = false;
            return;
        }

        // C-x is responsible for C-x C-c (exit), C-x C-s (save), and C-x u (undo).
        if self.ctrl_x_wait_flag {
            self.ctrl_x_wait_flag = false;
            // Hit "undo".
            if key.code == KeyCode::Char('u') && key.modifiers.is_empty() {
                self.buffer.editor.apply(Undo);
                self.buffer.editor.focus(&self.editor_area);
                return;
            }

            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    // Hit "save"
                    KeyCode::Char('s') => {
                        // If we've got file path the write the file.
                        if self.buffer.path.is_some() {
                            self.buffer.check_disk();
                            if self.buffer.obsolete {
                                self.overwrite_confirm = true;
                                self.status_bar_text =
                                    "File changed on disk => confirm overwrite".into();
                            } else {
                                match self.buffer.save() {
                                    Ok(()) => self.status_bar_text = "Saved".into(),
                                    Err(err) => {
                                        self.status_bar_text = format!("Save failed: {err}")
                                    }
                                }
                            }
                        } else {
                            // Else, manually ask for user input.
                            self.save_path_input = Some(String::new());
                            self.status_bar_text =
                                "Enter a file path => Enter to save, Esc to cancel".into();
                        }
                        return;
                    }

                    // Hit "exit"
                    KeyCode::Char('c') => {
                        self.exit_flag = true;
                        return;
                    }
                    _ => {}
                }
            }
        }

        // Reserve C-x for application commands rather than the editor's cut binding.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('x') {
            self.ctrl_x_wait_flag = true;
            self.status_bar_text.clear();
            return;
        }

        // Emacs-style pasteboard shortcuts.
        match (key.code, key.modifiers) {
            // C-M-_ => Redo. Shift could also be regarded as _.
            (KeyCode::Char('_'), modifiers)
                if modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.buffer.editor.apply(Redo)
            }
            (KeyCode::Char('-'), modifiers)
                if modifiers
                    .contains(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT) =>
            {
                self.buffer.editor.apply(Redo)
            }
            // M-w => Copy from kill ring.
            (KeyCode::Char('w'), KeyModifiers::ALT | KeyModifiers::CONTROL) => {
                if let Some(text) = self.buffer.editor.get_selection_text() {
                    let _ = self.buffer.editor.set_clipboard(&text);
                    self.kill_ring = Some(text);
                    if key.modifiers == KeyModifiers::CONTROL {
                        self.buffer.editor.apply(Delete);
                    }
                }
            }

            // C-y => "yanking" from kill ring.
            (KeyCode::Char('y'), KeyModifiers::CONTROL) => {
                if let Some(text) = &self.kill_ring {
                    self.buffer.editor.apply(InsertText { text: text.clone() });
                }
            }
            // C-c is a prefix handled with completion; C-v stays unbound.
            (KeyCode::Char('v'), KeyModifiers::CONTROL) => return,
            (KeyCode::Char('Z' | 'z'), modifiers)
                if modifiers == KeyModifiers::CONTROL | KeyModifiers::SHIFT =>
            {
                self.buffer.editor.apply(Redo)
            }
            // Shouldn't be reached: Unhandled keys
            _ => {
                if let Err(err) = self.buffer.editor.input(key, &self.editor_area) {
                    self.status_bar_text = format!("Editor error: {err}");
                }
                return;
            }
        }
        self.buffer.editor.focus(&self.editor_area);
    }
}

#[cfg(test)]
mod tests;
