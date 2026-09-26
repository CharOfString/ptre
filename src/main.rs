use std::io;

use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers,
};

use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};

// The state struct of editor
#[derive(Default)]
struct App {
    text: String,
    should_quit: bool,
}

// Implementation of the app struct
impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // The main loop: While the exit flag is false, keep ratatui running
        while !self.should_quit {
            // New frame
            terminal.draw(|frame| self.draw(frame))?;

            // Wait for event
            let event = event::read()?;

            // Update status
            if let Event::Key(key) = event {
                // The key release would result in counting the same part twice; ignoring it.
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                // Shortcut: exit key
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    if key.code == KeyCode::Char('q') {
                        self.should_quit = true;
                    }
                    continue;
                }

                match key.code {
                    // Directly write normal character into buffer unless you're pressing ALT key.
                    KeyCode::Char(c)
                        if !key.modifiers.contains(KeyModifiers::ALT) => {
                        self.text.push(c);
                    }

                    // Pressing ENTER pushes a \n to buffer.
                    KeyCode::Enter => self.text.push('\n'),

                    // Pressing BACKSPACE results a pop of last char in buffer.
                    KeyCode::Backspace => {
                        self.text.pop();
                    }

                    // Not handling anything other than those.
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        // We have buffer on top + status bar.
        let [editor_area, status_area] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        // Editor buffer.
        let editor = Paragraph::new(self.text.as_str())
            .block(Block::bordered().title(" Pointer "));

        frame.render_widget(editor, editor_area);

        // Status bar.
        let status = Paragraph::new(format!(
            " {} chars | C-Q: quit | DIRTY BUFFER",
            self.text.chars().count()
        ))
        .style(Style::default().fg(Color::Black).bg(Color::Cyan));

        // Init render
        frame.render_widget(status, status_area);
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}