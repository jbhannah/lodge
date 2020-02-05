use ignore::DirEntry;
use std::convert::TryFrom;
use std::ffi::OsString;
use std::io;
use std::iter::Iterator;
use std::ops::Deref;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    Empty(String),
    Io(io::Error),
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Empty(s.to_owned())
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Clone, Debug)]
pub struct Source {
    components: Vec<OsString>,
    target: PathBuf,
}

impl AsRef<Path> for Source {
    fn as_ref(&self) -> &Path {
        &self.target
    }
}

impl Deref for Source {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}

impl TryFrom<DirEntry> for Source {
    type Error = Error;

    fn try_from(entry: DirEntry) -> Result<Self, Self::Error> {
        let path = entry.path();
        let target = path.canonicalize()?;

        if target.is_dir() {
            return Err(Error::from("entry is a directory"));
        }

        let components = path
            .components()
            .rev()
            .take(entry.depth())
            .collect::<Vec<_>>();

        if components.is_empty() {
            return Err(Error::from("no components in path to link target"));
        }

        let components = components
            .iter()
            .rev()
            .map(|c| c.as_os_str().to_owned())
            .collect::<Vec<_>>();

        Ok(Self { components, target })
    }
}

impl Source {
    pub fn components(&self) -> std::slice::Iter<'_, std::ffi::OsString> {
        self.components.iter()
    }
}
