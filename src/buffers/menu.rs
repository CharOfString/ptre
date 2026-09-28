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

use super::cpp_checks::{Tool, settings::Settings};
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

const MENUS: [&str; 4] = ["FILE", "EDIT", "LSP", "C/C++"];
const CPP_MENU: usize = 3;

// C/C++ menu only shows if current active buffer is in C/C++ mode.
fn menus(cpp: bool) -> &'static [&'static str] {
    if cpp { &MENUS } else { &MENUS[..CPP_MENU] }
}

// What choosing a C/C++ menu entry does.
#[derive(Clone, Copy)]
enum CppAction {
    Toggle(Tool),
    Run(Tool),
}

// C/C++ menu entries. A "Run" entry only shows when its check can run.
fn cpp_entries(settings: &Settings) -> Vec<(&'static str, &'static str, CppAction)> {
    let tidy = if settings.tidy {
        "Clang-tidy: On"
    } else {
        "Clang-tidy: Off"
    };

    let cpplint = if settings.cpplint {
        "Cpplint: On"
    } else {
        "Cpplint: Off"
    };

    let mut entries = vec![
        (tidy, "", CppAction::Toggle(Tool::Tidy)),
        (cpplint, "", CppAction::Toggle(Tool::Cpplint)),
    ];

    if settings.tidy {
        entries.push(("Run Clang-tidy", "C-c t", CppAction::Run(Tool::Tidy)));
    }

    if settings.cpplint_ready() {
        entries.push(("Run Cpplint", "C-c l", CppAction::Run(Tool::Cpplint)));
    }
    entries
}

// Entries of one menu.
fn items(
    menu: usize,
    auto_completion: bool,
    cpp: Option<&Settings>,
) -> Vec<(&'static str, &'static str)> {
    match (menu, cpp) {
        (0, _) => FILE.to_vec(),
        (1, _) => EDIT.to_vec(),
        (2, _) => vec![
            (
                if auto_completion {
                    "Auto Complete: On"
                } else {
                    "Auto Complete: Off"
                },
                "C-c M-l",
            ),
            ("Complete", "M-/"),
        ],
        (CPP_MENU, Some(settings)) => cpp_entries(settings)
            .into_iter()
            .map(|(label, shortcut, _)| (label, shortcut))
            .collect(),
        _ => Vec::new(),
    }
}

#[derive(Default)]
pub(crate) struct Menu {
    pub(crate) active: Option<usize>,
    selected: usize,
}

impl Menu {
    pub(crate) fn draw_bar(&self, frame: &mut Frame, area: Rect, cpp: bool) {
        let base = Style::default().fg(LIGHT_BLUE);

        // App name & version.
        let mut spans = vec![
            Span::styled("  PTRE ", base.add_modifier(Modifier::BOLD)),
            Span::styled(format!("REL.{}  ", env!("CARGO_PKG_VERSION")), base),
        ];

        // App menu
        for (index, name) in menus(cpp).iter().enumerate() {
            let style = base.add_modifier(Modifier::BOLD);
            let indicator = if self.active == Some(index) {
                "❃ "
            } else {
                "  "
            };
            spans.push(Span::styled(indicator, base));
            spans.push(Span::styled(*name, style));
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
    pub(crate) fn draw_popup(
        &self,
        frame: &mut Frame,
        auto_completion: bool,
        cpp: Option<&Settings>,
    ) {
        let Some(active) = self.active else { return };
        let screen = frame.area();
        let items = items(active, auto_completion, cpp);

        // The C/C++ menu has no entries once the buffer is not C/C++.
        if screen.height <= 1 || screen.width == 0 || items.is_empty() {
            return;
        }

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
        // Each menu name takes its length plus four columns of indicator and padding.
        let before: usize = MENUS[..active].iter().map(|name| name.len() + 4).sum();
        let offset = (10 + env!("CARGO_PKG_VERSION").len() + before) as u16;
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

        let cpp = self.is_c_or_cpp();
        let count = items(active, true, cpp.then_some(&self.cpp_checks.settings)).len();

        // Close the C/C++ menu popup after the buffer changed language.
        if count == 0 {
            self.menu.active = None;
            return false;
        }

        let menu_count = menus(cpp).len();
        match key.code {
            KeyCode::Esc => self.menu.active = None,
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                let step = if key.code == KeyCode::Left {
                    menu_count - 1
                } else {
                    1
                };
                self.menu.active = Some((active + step) % menu_count);
                self.menu.selected = 0;
            }
            KeyCode::Down => self.menu.selected = (self.menu.selected + 1) % count,
            KeyCode::Up => self.menu.selected = (self.menu.selected + count - 1) % count,
            KeyCode::Enter => {
                let selected = self.menu.selected;
                self.menu.active = None;
                let ctrl = KeyModifiers::CONTROL;
                let alt = KeyModifiers::ALT;
                if active == 0 {
                    self.handle_key(KeyEvent::new(KeyCode::Char('x'), ctrl));
                    self.handle_key(KeyEvent::new(
                        KeyCode::Char(if selected == 0 { 's' } else { 'c' }),
                        ctrl,
                    ));
                } else if active == 2 {
                    if selected == 0 {
                        self.handle_key(KeyEvent::new(KeyCode::Char('c'), ctrl));
                        self.handle_key(KeyEvent::new(KeyCode::Char('l'), alt));
                    } else {
                        self.handle_key(KeyEvent::new(KeyCode::Char('/'), alt));
                    }
                } else if active == CPP_MENU {
                    // selected indexes the same entries the popup showed.
                    match cpp_entries(&self.cpp_checks.settings)[selected].2 {
                        CppAction::Toggle(tool) => self.toggle_cpp_check(tool),
                        CppAction::Run(tool) => self.run_cpp_check(tool),
                    }
                } else {
                    if selected == 0 {
                        self.handle_key(KeyEvent::new(KeyCode::Char('x'), ctrl));
                        self.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE));
                        return true;
                    }
                    let (code, modifiers) =
                        [('_', ctrl | alt), ('w', ctrl), ('w', alt), ('y', ctrl)][selected - 1];
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
    fn cpp_menu_hides_checks_that_cannot_run() {
        let mut settings = Settings::default();
        assert_eq!(menus(false), ["FILE", "EDIT", "LSP"]);
        assert_eq!(menus(true), ["FILE", "EDIT", "LSP", "C/C++"]);
        assert_eq!(items(2, true, Some(&settings)).len(), 2);
        let labels = |settings: &Settings| {
            let entries = items(CPP_MENU, true, Some(settings));
            entries
                .into_iter()
                .map(|(label, _)| label)
                .collect::<Vec<_>>()
        };
        assert_eq!(labels(&settings), ["Clang-tidy: Off", "Cpplint: Off"]);
        settings.tidy = true;
        assert_eq!(
            labels(&settings),
            ["Clang-tidy: On", "Cpplint: Off", "Run Clang-tidy"]
        );

        // Cpplint also needs an executable file as its path.
        settings.cpplint = true;
        for path in ["", "/ptre-missing/cpplint", "/", "Cargo.toml"] {
            settings.cpplint_path = path.into();
            assert_eq!(labels(&settings).len(), 3, "{path:?}");
        }
        settings.cpplint_path = std::env::current_exe().unwrap().display().to_string();
        assert_eq!(
            items(CPP_MENU, true, Some(&settings))[3],
            ("Run Cpplint", "C-c l")
        );
        assert!(items(CPP_MENU, true, None).is_empty());
        assert_eq!(items(0, true, Some(&settings)), FILE);
    }

    #[test]
    fn cpp_menu_shows_only_for_c_and_cpp_buffers() {
        let dir = std::env::temp_dir().join(format!("ptre-menu-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bar = |app: &mut App| {
            let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
            let screen = terminal.backend().buffer();
            (0..80).map(|x| screen[(x, 0)].symbol()).collect::<String>()
        };
        for (file, cpp) in [("main.c", true), ("main.cpp", true), ("notes.txt", false)] {
            let path = dir.join(file);
            std::fs::write(&path, "text").unwrap();
            let mut app = App::default();
            app.buffer.open(path).unwrap();
            assert_eq!(bar(&mut app).contains("C/C++"), cpp, "{file}");

            // Left from FILE wraps to the last menu shown.
            app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
            app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
            assert_eq!(app.menu.active, Some(if cpp { CPP_MENU } else { 2 }));
        }

        // With "Run Clang-tidy" hidden, the third entry runs Cpplint.
        let mut app = App::default();
        app.buffer.open(dir.join("main.c")).unwrap();
        let settings = &mut app.cpp_checks.settings;
        settings.cpplint = true;
        settings.cpplint_path = std::env::current_exe().unwrap().display().to_string();
        app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.status_bar_text, "Running Cpplint…");
        std::fs::remove_dir_all(&dir).unwrap();
    }

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
        for active in 0..3 {
            let mut app = App::default();
            app.menu.active = Some(active);
            let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
            let screen = terminal.backend().buffer();
            let popup_x = 10 + env!("CARGO_PKG_VERSION").len() as u16 + active as u16 * 8;
            let label: String = (0..80).map(|x| screen[(x, 0)].symbol()).collect();
            assert!(label.contains(&format!("❃ {}", MENUS[active])));
            assert_eq!(screen[(popup_x, 1)].symbol(), "╭");
            let popup_width = [18, 17, 31][active];
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
    fn lsp_menu_toggles_auto_completion_and_opens_popup() {
        let mut app = App::default();
        app.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::ALT));
        app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.menu.active, Some(2));
        app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.menu.active, Some(0));
        app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        assert_eq!(app.menu.active, Some(2));
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        let row = |app: &mut App, terminal: &mut Terminal<TestBackend>| {
            terminal.draw(|frame| app.draw(frame)).unwrap();
            (0..80)
                .map(|x| terminal.backend().buffer()[(x, 2)].symbol().to_owned())
                .collect::<String>()
        };
        assert!(row(&mut app, &mut terminal).contains("Auto Complete: On  C-c M-l"));

        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!app.completion.auto_enabled());
        assert!(app.menu.active.is_none());
        app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        assert!(row(&mut app, &mut terminal).contains("Auto Complete: Off  C-c M-l"));

        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        for c in "hello he".chars() {
            app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.handle_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.buffer.editor.get_content(), "hello hello");
    }

    #[test]
    fn tiny_terminals_do_not_panic() {
        for (width, height) in [(1, 1), (5, 2), (20, 4)] {
            let mut app = App::default();
            app.menu.active = Some(2);
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
        }
    }
}
