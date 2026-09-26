use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Paragraph},
};
use crate::app::App;

impl App {
    pub(crate) fn draw(&self, frame: &mut Frame) {
        // We have buffer on top + status bar.
        let [editor_area, status_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());

        // Editor buffer.
        let editor =
            Paragraph::new(self.buffer.text.as_str()).block(Block::bordered().title(" Pointer "));
        frame.render_widget(editor, editor_area);

        // Status bar.
        let status_text = if let Some(path_input) = &self.save_path_input {
            format!(
                " Save as: {path_input}█  |  Enter: save  Esc: cancel  |  {}",
                self.status
            )
        } else {
            let name = self
                .buffer
                .path
                .as_ref()
                .map_or_else(|| "[No Name]".to_owned(), |path| path.display().to_string());
            format!(
                " {name} | {} chars | C-x C-s: save | C-q: quit | {}",
                self.buffer.text.chars().count(),
                self.status
            )
        };
        let status =
            Paragraph::new(status_text).style(Style::default().fg(Color::Black).bg(Color::Cyan));
        // Init render
        frame.render_widget(status, status_area);
    }
}
