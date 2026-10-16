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

//! File tree on the left side, in the style of Emacs Treemacs (C-c c t).

use super::menu::LIGHT_BLUE;
use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, List, ListItem, ListState},
};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub(crate) struct Sidebar {
    pub(crate) visible: bool,
    pub(crate) focused: bool,
    // Set after C-c c, which waits for its next key.
    pub(crate) prefix: bool,
    root: PathBuf,
    // Folders that show their content.
    expanded: HashSet<PathBuf>,
    rows: Vec<Row>,
    // The selected row and the scroll position.
    list: ListState,
    // Set when the open file was selected; the next frame scrolls it to the middle.
    reveal: bool,
}

// The bar under the selected row, as the buttons in the confirm dialog.
const SELECTED: Color = Color::Rgb(60, 60, 68);

struct Row {
    path: PathBuf,
    depth: usize,
    is_dir: bool,
}

// The tree takes the left part of the area, at most 30 columns.
pub(crate) fn split(area: Rect) -> [Rect; 2] {
    let width = (area.width / 3).min(30);
    Layout::horizontal([Constraint::Length(width), Constraint::Min(0)]).areas(area)
}

fn name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into(),
    )
}

// Add the content of `dir` to `rows`, and the content of each open folder below it.
fn add_rows(dir: &Path, depth: usize, expanded: &HashSet<PathBuf>, rows: &mut Vec<Row>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut children: Vec<(bool, PathBuf)> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.file_name() != Some(".git".as_ref()))
        .map(|path| (path.is_dir(), path))
        .collect();
    // Folders first, then by name without case, as in Treemacs.
    children.sort_by_cached_key(|(is_dir, path)| (!is_dir, name(path).to_lowercase()));
    for (is_dir, path) in children {
        let open = is_dir && expanded.contains(&path);
        rows.push(Row {
            path: path.clone(),
            depth,
            is_dir,
        });
        if open {
            add_rows(&path, depth + 1, expanded, rows);
        }
    }
}

impl Sidebar {
    // Show the tree of `root`, with the folders above `file` open and `file` selected.
    fn show(&mut self, root: PathBuf, file: Option<&Path>) {
        if root != self.root {
            self.root = root;
            self.expanded.clear();
        }
        let file = file.filter(|file| file.starts_with(&self.root));
        if let Some(file) = file {
            let folders = file.ancestors().skip(1).take_while(|dir| *dir != self.root);
            self.expanded.extend(folders.map(Path::to_path_buf));
        }
        self.refresh();
        if let Some(index) = file.and_then(|file| self.rows.iter().position(|row| row.path == file))
        {
            self.list.select(Some(index));
            self.reveal = true;
        }
    }

    // Read the folders again and keep the same path selected when it still exists,
    // else the same row number.
    fn refresh(&mut self) {
        let selected = self.selected().map(|row| row.path.clone());
        self.rows.clear();
        add_rows(&self.root, 0, &self.expanded, &mut self.rows);
        let index = selected.and_then(|path| self.rows.iter().position(|row| row.path == path));
        let index = index.or(self.list.selected()).unwrap_or(0);
        let last = self.rows.len().checked_sub(1);
        self.list.select(last.map(|last| index.min(last)));
    }

    fn selected(&self) -> Option<&Row> {
        self.rows.get(self.list.selected()?)
    }

    // Move the selection by `step` rows, stopping at the first and the last row.
    fn step(&mut self, step: isize) {
        if let (Some(index), Some(last)) = (self.list.selected(), self.rows.len().checked_sub(1)) {
            self.list
                .select(Some(index.saturating_add_signed(step).min(last)));
        }
    }

    // Open or close the selected folder.
    fn fold(&mut self) {
        if let Some(row) = self.selected().filter(|row| row.is_dir) {
            let path = row.path.clone();
            if !self.expanded.remove(&path) {
                self.expanded.insert(path);
            }
            self.refresh();
        }
    }

    // Select the folder that holds the selected row.
    fn select_parent(&mut self) {
        let parent = self.selected().and_then(|row| row.path.parent());
        if let Some(index) = parent.and_then(|dir| self.rows.iter().position(|row| row.path == dir))
        {
            self.list.select(Some(index));
        }
    }

    pub(crate) fn draw(&mut self, frame: &mut Frame, area: Rect, nerd_font: bool) {
        let border = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(format!("─ {} ", name(&self.root)))
            .border_style(Style::default().fg(if self.focused {
                Color::Green
            } else {
                Color::White
            }));
        let items: Vec<ListItem> = self
            .rows
            .iter()
            .map(|row| {
                let open = self.expanded.contains(&row.path);
                let icon = match (row.is_dir, open, nerd_font) {
                    (true, false, false) => "▸ ",
                    (true, true, false) => "▾ ",
                    (true, false, true) => "\u{f07b} ",
                    (true, true, true) => "\u{f07c} ",
                    (false, _, false) => "  ",
                    (false, _, true) => "\u{f15b} ",
                };
                let style = if row.is_dir {
                    Style::default().fg(LIGHT_BLUE)
                } else {
                    Style::default()
                };
                let indent = "  ".repeat(row.depth);
                ListItem::new(Line::styled(
                    format!("{indent}{icon}{}", name(&row.path)),
                    style,
                ))
            })
            .collect();
        // The selected row gets a soft bar while the tree has focus, else bold text only.
        let mut highlight = Style::default().add_modifier(Modifier::BOLD);
        if self.focused {
            highlight = highlight.bg(SELECTED);
        }
        // Put a file that was just revealed in the middle of the tree.
        if std::mem::take(&mut self.reveal)
            && let Some(index) = self.list.selected()
        {
            let height = usize::from(area.height.saturating_sub(2));
            let offset = index.saturating_sub(height / 2);
            *self.list.offset_mut() = offset.min(self.rows.len().saturating_sub(height));
        }
        let list = List::new(items).block(border).highlight_style(highlight);
        frame.render_stateful_widget(list, area, &mut self.list);
    }
}

impl App {
    // Show the tree and give it focus, or hide it.
    pub(crate) fn toggle_sidebar(&mut self) {
        self.sidebar.visible = !self.sidebar.visible;
        self.sidebar.focused = self.sidebar.visible;
        if !self.sidebar.visible {
            return;
        }
        self.terminal.focused = false;
        // The tree starts at the project of the open file, or at the working directory.
        let file = self
            .buffer
            .path
            .as_deref()
            .and_then(|path| std::path::absolute(path).ok());
        let root = file
            .as_deref()
            .and_then(|file| crate::lsp::find_root(file).ok())
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        self.sidebar.show(root, file.as_deref());
    }

    // C-x o: focus the next visible pane, in the order tree, editor, terminal.
    fn focus_next_pane(&mut self) {
        let current = if self.sidebar.focused {
            0
        } else if self.terminal.focused {
            2
        } else {
            1
        };
        let visible = [self.sidebar.visible, true, self.terminal.visible];
        let next = (1..=3)
            .map(|step| (current + step) % 3)
            .find(|&pane| visible[pane])
            .unwrap_or(1);
        self.sidebar.focused = next == 0;
        self.terminal.focused = next == 2;
    }

    // Show the selected file in a tab, or open or close the selected folder.
    fn open_selected(&mut self) {
        let Some((path, is_dir)) = self
            .sidebar
            .selected()
            .map(|row| (row.path.clone(), row.is_dir))
        else {
            return;
        };
        if is_dir {
            self.sidebar.fold();
            return;
        }
        // A path below the working directory stays short in the status bar.
        let path = std::env::current_dir()
            .ok()
            .and_then(|dir| path.strip_prefix(dir).ok().map(Path::to_path_buf))
            .unwrap_or(path);
        if let Err(err) = self.open_tab(path) {
            self.status_bar_text = format!("Open failed: {err}");
            return;
        }
        self.sidebar.focused = false;
    }

    // Keys of C-c c, C-x o and the focused tree. Returns true when the key was used.
    pub(crate) fn handle_sidebar_key(&mut self, key: KeyEvent) -> bool {
        let plain = |c| key.code == KeyCode::Char(c) && key.modifiers.is_empty();
        if std::mem::take(&mut self.sidebar.prefix) {
            if plain('t') {
                self.toggle_sidebar();
            }
            return true;
        }
        if self.prompt_open() {
            return false;
        }
        if self.ctrl_x_wait_flag && plain('o') && (self.sidebar.visible || self.terminal.visible) {
            self.ctrl_x_wait_flag = false;
            self.focus_next_pane();
            return true;
        }
        if !self.sidebar.focused || self.keys_go_elsewhere() {
            return false;
        }

        let control = key.modifiers == KeyModifiers::CONTROL;
        match key.code {
            KeyCode::Down => self.sidebar.step(1),
            KeyCode::Up => self.sidebar.step(-1),
            KeyCode::Char('n') if plain('n') || control => self.sidebar.step(1),
            KeyCode::Char('p') if plain('p') || control => self.sidebar.step(-1),
            KeyCode::Tab => self.sidebar.fold(),
            KeyCode::Enter => self.open_selected(),
            KeyCode::Char('u') if plain('u') => self.sidebar.select_parent(),
            KeyCode::Char('g') if plain('g') => self.sidebar.refresh(),
            KeyCode::Char('q') if plain('q') => self.toggle_sidebar(),
            // Chords such as C-x C-s and the menu keys still work.
            _ if key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                || matches!(key.code, KeyCode::F(_)) =>
            {
                return false;
            }
            // Other keys do nothing, so they cannot change the buffer.
            _ => {}
        }
        true
    }
}

#[cfg(test)]
mod tests;
