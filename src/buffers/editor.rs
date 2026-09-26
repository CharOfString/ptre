use std::{fs, io, path::PathBuf};

// State struct of an editor buffer.
#[derive(Default)]
pub(crate) struct Buffer {
    pub(crate) text: String,
    pub(crate) path: Option<PathBuf>,
}

// Implementation of the editor buffer.
impl Buffer {
    pub(crate) fn save(&self) -> io::Result<()> {
        let path = self.path.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Buffer has no 'file path'.")
        })?;
        fs::write(path, &self.text)
    }

    pub(crate) fn save_as(&mut self, path: PathBuf) -> io::Result<()> {
        // Only update buffer path while write succeed
        fs::write(&path, &self.text)?;
        self.path = Some(path);
        Ok(())
    }
}
