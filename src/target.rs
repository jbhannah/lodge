use crate::{link, source::Source};
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    Target(String),
    Io(io::Error),
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Target(s.to_owned())
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Clone, Debug)]
pub struct Target(PathBuf);

impl AsRef<Path> for Target {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Deref for Target {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Target {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Target {
    pub fn new<P: AsRef<Path>>(p: P) -> Result<Self, Error> {
        let p = p.as_ref().to_owned();
        let metadata = p.metadata()?;

        if !metadata.is_dir() {
            Err(Error::from("target path is not a directory"))
        } else if metadata.permissions().readonly() {
            Err(Error::from("target path is not writable"))
        } else {
            Ok(Self(p))
        }
    }

    pub fn build_link(&self, src: &Source) -> Result<link::Link, link::Error> {
        let mut dst = self.clone();
        dst.extend(src.components_iter());
        link::Link::new(src, dst)
    }
}
