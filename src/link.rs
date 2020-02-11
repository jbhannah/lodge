use std::fs::{remove_file, DirBuilder};
use std::io;
use std::ops::Deref;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    Skip(String),
    Io(io::Error),
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Skip(s.to_owned())
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Clone, Debug)]
pub struct Link {
    dst: PathBuf,
    src: PathBuf,
    rm: bool,
}

impl AsRef<Path> for Link {
    fn as_ref(&self) -> &Path {
        &self.dst
    }
}

impl Deref for Link {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.dst
    }
}

impl Link {
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<Self, Error> {
        let src = src.as_ref();
        let dst = dst.as_ref();

        let rm = {
            let src_meta = src.metadata()?;
            if let Ok(dst_meta) = dst.symlink_metadata() {
                if dst_meta.file_type().is_symlink() {
                    match dst.read_link() {
                        Ok(p) => {
                            if p.as_path() == src {
                                return Err(Error::from("target already links to source"));
                            } else {
                                true
                            }
                        }
                        _ => true,
                    }
                } else if dst.exists() {
                    if dst_meta.file_type().is_file()
                        && dst_meta.file_type() == src_meta.file_type()
                        && dst_meta.len() == src_meta.len()
                    {
                        true
                    } else {
                        return Err(Error::from("file at target path differs from source file"));
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };

        Ok(Self {
            dst: dst.to_owned(),
            src: src.to_owned(),
            rm,
        })
    }

    pub fn mklink(&self) -> io::Result<()> {
        if let Some(parent) = self.dst.parent() {
            DirBuilder::new().recursive(true).create(parent)?;
        }

        if self.rm {
            remove_file(self.dst.as_path())?;
        }

        println!("Linking {} -> {}", self.dst.display(), self.src.display());
        symlink(&self.src, &self.dst)
    }
}
