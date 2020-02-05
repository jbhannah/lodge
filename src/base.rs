use std::io;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    Base(String),
    Io(io::Error),
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Base(s.to_owned())
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Clone, Debug)]
pub struct Base(PathBuf);

impl AsRef<Path> for Base {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Deref for Base {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Base {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Base {
    pub fn new<P: AsRef<Path>>(p: P) -> Result<Self, Error> {
        let p = p.as_ref().to_owned();
        let metadata = p.metadata()?;

        if !metadata.is_dir() {
            Err(Error::from("base path is not a directory"))
        } else if metadata.permissions().readonly() {
            Err(Error::from("base path is not writable"))
        } else {
            Ok(Self(p))
        }
    }
}
