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

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph},
};
use std::path::Path;

// The words of a yes or no dialog.
pub(super) struct Confirm {
    title: &'static str,
    question: &'static str,
    warning: &'static str,
    action: &'static str,
    cancel: &'static str,
    short: &'static str,
}

pub(super) const OVERWRITE: Confirm = Confirm {
    title: "Confirm overwrite",
    question: "File has been changed on disk. Overwrite it?",
    warning: "Saving will replace the external changes.",
    action: " Overwriting",
    cancel: "Cancel keeps both your file and buffer unchanged.",
    short: "File changed on disk.",
};

pub(super) const CLOSE: Confirm = Confirm {
    title: "Confirm close",
    question: "Buffer has unsaved changes. Close it?",
    warning: "Closing will lose the unsaved changes.",
    action: " Close without saving",
    cancel: "Cancel keeps the tab open.",
    short: "Unsaved changes.",
};

pub(super) fn draw_confirm(frame: &mut Frame, path: Option<&Path>, words: &Confirm) {
    let area = frame.area();
    let bg = Color::Rgb(28, 28, 32);
    let fg = Color::Rgb(225, 222, 216);
    let muted = Color::Rgb(140, 138, 145);
    let amber = Color::Rgb(232, 181, 106);
    let red = Color::Rgb(235, 139, 139);

    // Dim the background layer.
    frame.buffer_mut().set_style(
        area,
        Style::default()
            .fg(Color::Rgb(85, 84, 91))
            .bg(Color::Rgb(17, 17, 20))
            .add_modifier(Modifier::DIM),
    );
    let width = area.width.saturating_sub(4).min(64);
    let height = area.height.saturating_sub(2).min(13);
    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, popup);
    let panel = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(94, 79, 60)))
        .style(Style::default().fg(fg).bg(bg));
    let inner = panel.inner(popup);
    frame.render_widget(panel, popup);

    let filename = path
        .and_then(Path::file_name)
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "[No Name]".into());
    let buttons = Line::from(vec![
        Span::styled(
            " N / Esc ",
            Style::default().fg(fg).bg(Color::Rgb(60, 60, 68)).bold(),
        ),
        Span::styled(" Keep editing   ", Style::default().fg(fg)),
        Span::styled(" Y/ENTER ", Style::default().fg(bg).bg(red).bold()),
        Span::styled(words.action, Style::default().fg(red).bg(bg).bold()),
    ]);
    let lines = if inner.height >= 9 && inner.width >= 48 {
        vec![
            Line::from(""),
            Line::styled(
                format!("!  {}", words.title),
                Style::default().fg(amber).bold(),
            ),
            Line::from(""),
            Line::styled(filename, Style::default().fg(fg).bold()),
            Line::styled(words.question, Style::default().fg(fg)),
            Line::styled(words.warning, Style::default().fg(muted)),
            Line::from(""),
            buttons,
            Line::styled(words.cancel, Style::default().fg(muted)),
        ]
    } else {
        vec![
            Line::styled(words.title, Style::default().fg(amber).bold()),
            Line::from(words.short),
            Line::styled("Y/ENTER: confirm", Style::default().fg(red)),
            Line::from("N / Esc: cancel"),
        ]
    };
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn overwrite_border_has_rounded_corners() {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|frame| draw_confirm(frame, None, &OVERWRITE))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let x = 8;
        let y = 5;
        assert_eq!(buffer[(x, y)].symbol(), "╭");
        assert_eq!(buffer[(x + 63, y)].symbol(), "╮");
        assert_eq!(buffer[(x, y + 12)].symbol(), "╰");
        assert_eq!(buffer[(x + 63, y + 12)].symbol(), "╯");
        assert_eq!(buffer[(x + 1, y)].symbol(), "─");
        assert_eq!(buffer[(x, y + 1)].symbol(), "│");
    }
}
