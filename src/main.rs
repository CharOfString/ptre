mod app;
mod buffers;

use std::io;
use app::App;

// Main app of Pointer.
fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
