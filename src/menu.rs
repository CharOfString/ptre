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

use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, List, ListItem, ListState, Paragraph},
};

pub(crate) const LIGHT_BLUE: Color = Color::Rgb(155, 205, 245);
const FILE: &[(&str, &str)] = &[("Save", "C-x C-s"), ("Quit", "C-x C-c")];
const EDIT: &[(&str, &str)] = &[
    ("Undo", "C-x u"),
    ("Redo", "C-M-_"),
    ("Cut", "C-w"),
    ("Copy", "M-w"),
    ("Paste", "C-y"),
];

#[derive(Default)]
pub(crate) struct Menu {
    pub(crate) active: Option<usize>,
    selected: usize,
}

impl Menu {
    pub(crate) fn draw_bar(&self, frame: &mut Frame, area: Rect) {
        let base = Style::default().fg(LIGHT_BLUE);

        // App name & version.
        let mut spans = vec![
            Span::styled("  PTRE ", base.add_modifier(Modifier::BOLD)),
            Span::styled(format!("REL.{}  ", env!("CARGO_PKG_VERSION")), base),
        ];

        // App menu
        for (index, name) in ["FILE", "EDIT"].into_iter().enumerate() {
            let style = base.add_modifier(Modifier::BOLD);
            let indicator = if self.active == Some(index) {
                "❃ "
            } else {
                "  "
            };
            spans.push(Span::styled(indicator, base));
            spans.push(Span::styled(name, style));
            spans.push(Span::styled("  ", base));
        }

        if area.height == 0 || area.width == 0 {
            return;
        }

        let border = if area.width == 1 {
            "╭".to_owned()
        } else {
            format!("╭{}╮", "─".repeat(usize::from(area.width - 2)))
        };

        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(base),
            Rect::new(area.x, area.y, area.width, 1),
        );

        if area.height > 1 {
            frame.render_widget(
                Paragraph::new(border).style(base),
                Rect::new(area.x, area.y + 1, area.width, 1),
            );
        }
    }

    // Popup menu.
    pub(crate) fn draw_popup(&self, frame: &mut Frame) {
        let Some(active) = self.active else { return };
        let screen = frame.area();
        if screen.height <= 1 || screen.width == 0 {
            return;
        }

        let items = if active == 0 { FILE } else { EDIT };
        let label_width = items
            .iter()
            .map(|(label, _)| label.len())
            .max()
            .unwrap_or(0);

        let shortcut_width = items
            .iter()
            .map(|(_, shortcut)| shortcut.len())
            .max()
            .unwrap_or(0);

        // Borders, selection indicator, side padding, and a two-column gap.
        let width = (label_width + shortcut_width + 7) as u16;
        let width = width.min(screen.width);
        let offset = 10 + env!("CARGO_PKG_VERSION").len() as u16 + active as u16 * 8;
        let area = Rect::new(
            screen.x + offset.min(screen.width.saturating_sub(width)),
            screen.y + 1,
            width,
            (items.len() as u16 + 2).min(screen.height - 1),
        );

        let rows: Vec<_> = items
            .iter()
            .map(|(label, shortcut)| {
                ListItem::new(format!(
                    " {label:<label_width$}  {shortcut:>shortcut_width$} "
                ))
            })
            .collect();

        let list = List::new(rows)
            .style(Style::default().fg(LIGHT_BLUE))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(LIGHT_BLUE)),
            )
            .highlight_style(Style::default().fg(LIGHT_BLUE).add_modifier(Modifier::BOLD))
            .highlight_symbol("·");
        frame.render_widget(Clear, area);
        frame.render_stateful_widget(
            list,
            area,
            &mut ListState::default().with_selected(Some(self.selected)),
        );
    }
}

impl App {
    // Invoke and activated menu.
    pub(crate) fn handle_menu_key(&mut self, key: KeyEvent) -> bool {
        let requested = match (key.code, key.modifiers) {
            (KeyCode::F(10), _) => Some(0),
            (KeyCode::Char('f' | 'F'), KeyModifiers::ALT) => Some(0),
            (KeyCode::Char('e' | 'E'), KeyModifiers::ALT) => Some(1),
            _ => None,
        };

        if let Some(index) = requested {
            self.menu.active = if self.menu.active == Some(index) {
                None
            } else {
                Some(index)
            };
            self.menu.selected = 0;
            return true;
        }

        let Some(active) = self.menu.active else {
            return false;
        };

        let count = if active == 0 { FILE.len() } else { EDIT.len() };
        match key.code {
            KeyCode::Esc => self.menu.active = None,
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                self.menu.active = Some(1 - active);
                self.menu.selected = 0;
            }
            KeyCode::Down => self.menu.selected = (self.menu.selected + 1) % count,
            KeyCode::Up => self.menu.selected = (self.menu.selected + count - 1) % count,
            KeyCode::Enter => {
                let selected = self.menu.selected;
                self.menu.active = None;
                let ctrl = KeyModifiers::CONTROL;
                if active == 0 {
                    self.handle_key(KeyEvent::new(KeyCode::Char('x'), ctrl));
                    self.handle_key(KeyEvent::new(
                        KeyCode::Char(if selected == 0 { 's' } else { 'c' }),
                        ctrl,
                    ));
                } else {
                    if selected == 0 {
                        self.handle_key(KeyEvent::new(KeyCode::Char('x'), ctrl));
                        self.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE));
                        return true;
                    }
                    let (code, modifiers) = [
                        ('_', ctrl | KeyModifiers::ALT),
                        ('w', ctrl),
                        ('w', KeyModifiers::ALT),
                        ('y', ctrl),
                    ][selected - 1];
                    self.handle_key(KeyEvent::new(KeyCode::Char(code), modifiers));
                }
            }
            _ => {}
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn renders_light_blue_menu_and_separator() {
        let mut app = App::default();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let screen = terminal.backend().buffer();
        let title: String = (0..80).map(|x| screen[(x, 0)].symbol()).collect();
        assert!(title.contains(&format!("PTRE REL.{}", env!("CARGO_PKG_VERSION"))));
        assert!(title.contains("FILE"));
        assert!(title.contains("EDIT"));
        assert_eq!(screen[(0, 0)].bg, Color::Reset);
        assert_eq!(screen[(2, 0)].fg, LIGHT_BLUE);
        assert_eq!(screen[(0, 1)].symbol(), "╭");
        assert_eq!(screen[(79, 1)].symbol(), "╮");
        for x in 1..79 {
            assert_eq!(screen[(x, 1)].symbol(), "─");
        }
        for x in 0..80 {
            assert!(!screen[(x, 0)].modifier.contains(Modifier::UNDERLINED));
            assert_eq!(screen[(x, 1)].fg, LIGHT_BLUE);
            assert_eq!(screen[(x, 1)].bg, Color::Reset);
        }
        app.menu.active = Some(0);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let screen = terminal.backend().buffer();
        let popup_x = 10 + env!("CARGO_PKG_VERSION").len() as u16;
        assert_eq!(screen[(popup_x, 1)].bg, Color::Reset);
        assert_eq!(screen[(popup_x, 1)].fg, LIGHT_BLUE);
    }

    #[test]
    fn active_menu_and_popup_use_markers_without_underlines() {
        for active in 0..2 {
            let mut app = App::default();
            app.menu.active = Some(active);
            let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
            let screen = terminal.backend().buffer();
            let popup_x = 10 + env!("CARGO_PKG_VERSION").len() as u16 + active as u16 * 8;
            let label: String = (0..80).map(|x| screen[(x, 0)].symbol()).collect();
            assert!(label.contains(if active == 0 { "❃ FILE" } else { "❃ EDIT" }));
            assert_eq!(screen[(popup_x, 1)].symbol(), "╭");
            let popup_width = if active == 0 { 18 } else { 17 };
            assert_eq!(screen[(popup_x + popup_width - 1, 1)].symbol(), "╮");
            assert_eq!(screen[(popup_x + 1, 2)].symbol(), "·");
            assert_eq!(screen[(popup_x + 1, 3)].symbol(), " ");
            for y in 0..4 {
                for x in 0..80 {
                    let cell = &screen[(x, y)];
                    assert!(
                        !cell
                            .modifier
                            .intersects(Modifier::REVERSED | Modifier::UNDERLINED)
                    );
                    assert_eq!(cell.bg, Color::Reset);
                }
            }
            app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            terminal.draw(|frame| app.draw(frame)).unwrap();
            let screen = terminal.backend().buffer();
            assert_eq!(screen[(popup_x + 1, 2)].symbol(), " ");
            assert_eq!(screen[(popup_x + 1, 3)].symbol(), "·");
        }
    }

    #[test]
    fn menu_consumes_typing_and_save_opens_prompt() {
        let mut app = App::default();
        app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        assert_eq!(app.buffer.editor.get_content(), "");
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(app.save_path_input.is_some());
        assert!(app.menu.active.is_none());
    }

    #[test]
    fn edit_menu_undo_and_escape() {
        let mut app = App::default();
        app.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::ALT));
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.buffer.editor.get_content(), "");
        app.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::ALT));
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.buffer.editor.get_content(), "a");
        app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.menu.active, Some(1));
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(app.menu.active.is_none());
    }

    #[test]
    fn tiny_terminals_do_not_panic() {
        for (width, height) in [(1, 1), (5, 2), (20, 4)] {
            let mut app = App::default();
            app.menu.active = Some(1);
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
        }
    }
}
