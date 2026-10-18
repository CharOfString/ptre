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

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType},
};
use std::{
    io::{self, Read, Write},
    path::Path,
    sync::mpsc::{self, Receiver},
};

// Keep the shell alive when its pane is hidden.
#[derive(Default)]
pub(crate) struct Terminal {
    session: Option<Session>,
    pub(crate) visible: bool,
    pub(crate) focused: bool,
}

struct Session {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    output: Receiver<Vec<u8>>,
    parser: vt100::Parser<Title>,
    exited: bool,
}

// Keep the window title that the shell sets with OSC 0 or OSC 2.
#[derive(Default)]
struct Title(String);

impl vt100::Callbacks for Title {
    fn set_window_title(&mut self, _: &mut vt100::Screen, title: &[u8]) {
        // Drop control characters so they cannot break the border.
        let title = String::from_utf8_lossy(title);
        self.0 = title.chars().filter(|c| !c.is_control()).collect();
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Session {
    fn start(cwd: Option<&Path>) -> io::Result<Self> {
        let pair = native_pty_system()
            .openpty(size(24, 80))
            .map_err(io::Error::other)?;
        let mut command =
            CommandBuilder::new(std::env::var_os("SHELL").unwrap_or_else(|| "/bin/sh".into()));
        command.env("TERM", "xterm-256color");
        if let Some(cwd) = cwd {
            command.cwd(cwd);
        }
        let mut reader = pair.master.try_clone_reader().map_err(io::Error::other)?;
        let writer = pair.master.take_writer().map_err(io::Error::other)?;
        let child = pair
            .slave
            .spawn_command(command)
            .map_err(io::Error::other)?;
        drop(pair.slave);
        // Bound queued output so a noisy command cannot exhaust memory.
        let (sender, output) = mpsc::sync_channel(64);
        std::thread::spawn(move || {
            let mut bytes = [0; 8192];
            loop {
                match reader.read(&mut bytes) {
                    Ok(0) => break,
                    Ok(count) => {
                        if sender.send(bytes[..count].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            master: pair.master,
            writer,
            child,
            output,
            parser: vt100::Parser::new_with_callbacks(24, 80, 1000, Title::default()),
            exited: false,
        })
    }
}

fn size(rows: u16, cols: u16) -> PtySize {
    PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    }
}

// Reserve enough height for both panes; short windows use a side split.
pub(crate) fn split(area: Rect) -> [Rect; 2] {
    if area.height >= 28 {
        Layout::vertical([Constraint::Percentage(65), Constraint::Percentage(35)]).areas(area)
    } else {
        Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)]).areas(area)
    }
}

impl Terminal {
    pub(crate) fn toggle(&mut self, cwd: Option<&Path>) -> io::Result<()> {
        if !self.visible && self.session.as_ref().is_none_or(|session| session.exited) {
            self.session = Some(Session::start(cwd)?);
        }
        self.visible = !self.visible;
        self.focused = self.visible;
        Ok(())
    }

    pub(crate) fn poll(&mut self) {
        if let Some(session) = &mut self.session {
            // Limit work per frame so continuous output does not block input.
            for bytes in session.output.try_iter().take(64) {
                session.parser.process(&bytes);
            }
            if matches!(session.child.try_wait(), Ok(Some(_))) {
                session.exited = true;
            }
        }
    }

    pub(crate) fn input(&mut self, key: KeyEvent) -> io::Result<()> {
        let Some(session) = &self.session else {
            return Ok(());
        };
        let bytes = encode_key(key, session.parser.screen().application_cursor());
        self.write(&bytes)
    }

    // Send pasted text. Mark its start and end when the program asked for that.
    pub(crate) fn paste(&mut self, text: &str) -> io::Result<()> {
        let Some(session) = &self.session else {
            return Ok(());
        };
        if session.parser.screen().bracketed_paste() {
            self.write(format!("\x1b[200~{text}\x1b[201~").as_bytes())
        } else {
            self.write(text.as_bytes())
        }
    }

    fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        if let Some(session) = &mut self.session {
            session.parser.screen_mut().set_scrollback(0);
            if session.exited {
                return Ok(());
            }
            session.writer.write_all(bytes)?;
            session.writer.flush()?;
        }
        Ok(())
    }

    pub(crate) fn draw(&mut self, frame: &mut Frame, area: Rect) -> io::Result<()> {
        let session = self.session.as_ref();
        let exited = session.is_some_and(|session| session.exited);
        let offset = session.map_or(0, |s| s.parser.screen().scrollback());
        // Show the shell title like the editor shows its file name.
        let name = session
            .map(|s| s.parser.callbacks().0.as_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("Terminal");
        let mut title = format!("─ {name}");
        if exited {
            title.push_str(" · EXITED");
        }
        if offset > 0 {
            title.push_str(&format!(" · HISTORY {offset}"));
        }
        title.push(' ');
        let border = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(title)
            .border_style(Style::default().fg(if self.focused {
                Color::Green
            } else {
                Color::White
            }));
        let inner = border.inner(area);
        frame.render_widget(border, area);
        let Some(session) = &mut self.session else {
            return Ok(());
        };
        if inner.is_empty() {
            return Ok(());
        }
        if session.parser.screen().size() != (inner.height, inner.width) {
            session
                .master
                .resize(size(inner.height, inner.width))
                .map_err(io::Error::other)?;
            session
                .parser
                .screen_mut()
                .set_size(inner.height, inner.width);
        }
        let screen = session.parser.screen();
        for row in 0..inner.height {
            for col in 0..inner.width {
                if let Some(cell) = screen.cell(row, col) {
                    let mut style = Style::default()
                        .fg(color(cell.fgcolor()))
                        .bg(color(cell.bgcolor()));
                    for (enabled, modifier) in [
                        (cell.bold(), Modifier::BOLD),
                        (cell.dim(), Modifier::DIM),
                        (cell.italic(), Modifier::ITALIC),
                        (cell.underline(), Modifier::UNDERLINED),
                        (cell.inverse(), Modifier::REVERSED),
                    ] {
                        if enabled {
                            style = style.add_modifier(modifier);
                        }
                    }
                    let target = &mut frame.buffer_mut()[(inner.x + col, inner.y + row)];
                    target.set_symbol(if cell.has_contents() {
                        cell.contents()
                    } else {
                        " "
                    });
                    target.set_style(style);
                }
            }
        }
        let (row, col) = screen.cursor_position();
        if self.focused
            && !exited
            && screen.scrollback() == 0
            && !screen.hide_cursor()
            && row < inner.height
            && col < inner.width
        {
            frame.set_cursor_position((inner.x + col, inner.y + row));
        }
        Ok(())
    }
}

// Keep ordinary PageUp and PageDown available to programs running in the PTY.
fn scroll_history(screen: &mut vt100::Screen, key: KeyEvent) -> bool {
    if screen.alternate_screen() {
        return false;
    }
    let up = match (key.code, key.modifiers) {
        (KeyCode::Char('v'), KeyModifiers::ALT) | (KeyCode::PageUp, KeyModifiers::SHIFT) => true,
        (KeyCode::Char('v'), KeyModifiers::CONTROL) | (KeyCode::PageDown, KeyModifiers::SHIFT) => {
            false
        }
        _ => return false,
    };
    // Overlap one row to make consecutive pages easier to follow.
    let page = usize::from(screen.size().0.saturating_sub(1).max(1));
    let offset = screen.scrollback();
    screen.set_scrollback(if up {
        offset.saturating_add(page)
    } else {
        offset.saturating_sub(page)
    });
    true
}

fn color(value: vt100::Color) -> Color {
    match value {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(index) => Color::Indexed(index),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

fn encode_key(key: KeyEvent, application_cursor: bool) -> Vec<u8> {
    let mut bytes = match key.code {
        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => match c {
            'a'..='z' | 'A'..='Z' | '@'..='_' => vec![(c as u8) & 0x1f],
            ' ' => vec![0],
            '?' => vec![127],
            _ => return Vec::new(),
        },
        KeyCode::Char(c) => c.to_string().into_bytes(),
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![127],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Esc => vec![27],
        KeyCode::Up
        | KeyCode::Down
        | KeyCode::Right
        | KeyCode::Left
        | KeyCode::Home
        | KeyCode::End => {
            let suffix = match key.code {
                KeyCode::Up => 'A',
                KeyCode::Down => 'B',
                KeyCode::Right => 'C',
                KeyCode::Left => 'D',
                KeyCode::Home => 'H',
                _ => 'F',
            };
            format!("\x1b{}{suffix}", if application_cursor { 'O' } else { '[' }).into_bytes()
        }
        KeyCode::Insert => b"\x1b[2~".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        KeyCode::F(n @ 1..=4) => vec![27, b'O', b'P' + n - 1],
        KeyCode::F(n @ 5..=12) => {
            let code = [15, 17, 18, 19, 20, 21, 23, 24][usize::from(n - 5)];
            format!("\x1b[{code}~").into_bytes()
        }
        _ => return Vec::new(),
    };
    if key.modifiers.contains(KeyModifiers::ALT) {
        bytes.insert(0, 27);
    }
    bytes
}

impl crate::app::App {
    // True while a dialog, a prompt or the menu reads the keys.
    pub(crate) fn prompt_open(&self) -> bool {
        self.cpp_check_modal()
            || self.overwrite_confirm
            || self.tabs.close_confirm
            || self.save_path_input.is_some()
            || self.sidebar.project_input.is_some()
            || self.search.reads_input()
            || self.goto_line.is_open()
            || self.menu.active.is_some()
    }

    pub(crate) fn handle_terminal_key(&mut self, key: KeyEvent) -> bool {
        // Modal editor prompts retain their input and focus.
        if self.prompt_open() {
            return false;
        }
        let plain = |c| key.code == KeyCode::Char(c) && key.modifiers.is_empty();
        let ctrl_x = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL);

        // C-c t t opens or hides the terminal; C-c t c runs clang-tidy.
        if self.ctrl_c_t_wait_flag {
            self.ctrl_c_t_wait_flag = false;
            if plain('t') {
                let cwd = self
                    .buffer
                    .path
                    .as_deref()
                    .and_then(Path::parent)
                    .filter(|path| !path.as_os_str().is_empty());
                if let Err(err) = self.terminal.toggle(cwd) {
                    self.status_bar_text = format!("Terminal failed: {err}");
                }
                // Only one pane has focus.
                self.sidebar.focused &= !self.terminal.focused;
            } else if plain('c') {
                self.run_cpp_check(crate::buffers::cpp_checks::Tool::Tidy);
            }
            return true;
        }

        if self.terminal.focused {
            if !self.ctrl_x_wait_flag
                && let Some(session) = &mut self.terminal.session
                && scroll_history(session.parser.screen_mut(), key)
            {
                return true;
            }
            // Hold C-x back so C-x o can leave the terminal.
            if key == ctrl_x && !self.ctrl_x_wait_flag {
                self.ctrl_x_wait_flag = true;
                return true;
            }
            // Any other key after C-x goes to the shell together with C-x.
            let mut result = Ok(());
            if std::mem::take(&mut self.ctrl_x_wait_flag) {
                result = self.terminal.input(ctrl_x);
            }
            if let Err(err) = result.and_then(|()| self.terminal.input(key)) {
                self.status_bar_text = format!("Terminal input failed: {err}");
            }
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests;
