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
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, List, ListItem, ListState},
};

const MAX_ROWS: usize = 8;
const MAX_WIDTH: u16 = 60;

impl App {
    // Draw the popup just below the word being completed, or above it near the bottom.
    pub(crate) fn draw_completion(&self, frame: &mut Frame) {
        let completion = &self.completion;
        // A popup opened by typing stays hidden until there is something to show.
        if !completion.active || (completion.visible.is_empty() && !completion.manual) {
            return;
        }
        let Some((x, y)) = self.buffer.editor.get_visible_cursor(&self.editor_area) else {
            return;
        };
        let dim = Style::default().fg(Color::DarkGray);
        let rows: Vec<ListItem> = if completion.visible.is_empty() {
            vec![ListItem::new(Span::styled("Loading…", dim))]
        } else {
            completion
                .visible
                .iter()
                .map(|&index| {
                    let candidate = &completion.candidates[index];
                    let mut line = Line::from(candidate.label.as_str());
                    if let Some(detail) = &candidate.detail {
                        line.push_span(Span::styled(format!("  {detail}"), dim));
                    }
                    ListItem::new(line)
                })
                .collect()
        };

        let area = frame.area();
        let width = rows.iter().map(ListItem::width).max().unwrap_or(0) as u16 + 2;
        let width = width.min(MAX_WIDTH).min(area.width);
        let height = rows.len().min(MAX_ROWS) as u16 + 2;
        // Line the labels up with the start of the word (the border takes one column).
        let typed = self
            .buffer
            .editor
            .get_cursor()
            .saturating_sub(completion.word_start);
        let x = x
            .saturating_sub(typed as u16 + 1)
            .min(area.right().saturating_sub(width));
        let y = if y + 1 + height <= area.bottom() {
            y + 1
        } else {
            y.saturating_sub(height)
        };
        let popup = Rect::new(x, y, width, height).intersection(area);

        let list = List::new(rows)
            .block(Block::bordered().border_type(BorderType::Rounded))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        let mut state = ListState::default().with_selected(Some(completion.selected));
        frame.render_widget(Clear, popup);
        frame.render_stateful_widget(list, popup, &mut state);
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::type_text;
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn screen_rows(app: &mut App) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let screen = terminal.backend().buffer();
        (0..24)
            .map(|y| (0..80).map(|x| screen[(x, y)].symbol()).collect())
            .collect()
    }

    #[test]
    fn popup_is_drawn_under_the_word() {
        let mut app = App::default();
        type_text(&mut app, "alpha alps\nal");
        let rows = screen_rows(&mut app);
        let (x, y) = app
            .buffer
            .editor
            .get_visible_cursor(&app.editor_area)
            .unwrap();
        let (x, y) = (usize::from(x), usize::from(y));
        assert!(rows[y + 2].contains("│alps"), "{}", rows[y + 2]);
        assert!(rows[y + 3].contains("│alpha"));
        assert_eq!(rows[y + 2].chars().nth(x - 2), Some('a'));
    }

    #[test]
    fn pending_popup_shows_loading_only_when_opened_by_hand() {
        for manual in [false, true] {
            let mut app = App::default();
            app.completion.open(manual);
            app.completion.waiting = Some(0);
            let loading = screen_rows(&mut app)
                .iter()
                .any(|row| row.contains("Loading…"));
            assert_eq!(loading, manual);
        }
    }
}
