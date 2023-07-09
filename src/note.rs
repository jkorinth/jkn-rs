use std::path::{Path, PathBuf};
use std::io::{self, BufRead};
use std::fs;

#[derive(Debug)]
pub struct Note {
    path: PathBuf,
}

impl Note {
    pub fn from(path: &Path) -> Self {
        Note {
            path: PathBuf::from(path),
        }
    }

    pub fn summary(&self) -> Result<String, io::Error> {
        let f = fs::File::open(&self.path)?;
        let mut br = io::BufReader::new(f);
        let mut first_line = String::new();
        let _ = br.read_line(&mut first_line);
        Ok(first_line)
    }
}
