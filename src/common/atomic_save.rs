//! Adapted from https://github.com/untitaker/rust-atomicwrites/blob/master/src/lib.rs.

use std::error::Error as ErrorTrait;
use std::fmt;
use std::fs;
use std::io;
use std::path;

pub use OverwriteBehavior::{AllowOverwrite, DisallowOverwrite};

/// Whether to allow overwriting if the target file exists.
#[derive(Clone, Copy)]
pub enum OverwriteBehavior {
    /// Overwrite files silently.
    AllowOverwrite,

    /// Don't overwrite files. `AtomicFile.write` will raise errors for such conditions only after
    /// you've already written your data.
    DisallowOverwrite,
}

/// Whether to ensure durability after a system crash (guaranteed to contain the new data).
/// Regardless of the option you pick, the atomic write will be consistent after a crash
/// (will never contain partially-written data).
#[derive(Clone, Copy)]
pub enum Durability {
    /// Faster, ensures either old or new file contents (but not half-written data)
    /// will be present after system crash.
    DontSyncDir,
    /// Slower, ensures new file contents will be present after system crash.
    /// Not possible on Windows.
    SyncDir,
}

/// Represents an error raised by `AtomicFile.write`.
#[derive(Debug)]
pub enum AtomicSaveError<E> {
    /// The error originated in the library itself, while it was either creating a temporary file
    /// or moving the file into place.
    Internal(io::Error),
    /// The error originated in the user-supplied callback.
    User(E),
}

/// If your callback returns a `std::io::Error`, you can unwrap this type to `std::io::Error`.
impl From<AtomicSaveError<io::Error>> for io::Error {
    fn from(e: AtomicSaveError<io::Error>) -> Self {
        match e {
            AtomicSaveError::Internal(x) => x,
            AtomicSaveError::User(x) => x,
        }
    }
}

impl<E: fmt::Display> fmt::Display for AtomicSaveError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AtomicSaveError::Internal(ref e) => e.fmt(f),
            AtomicSaveError::User(ref e) => e.fmt(f),
        }
    }
}

impl<E: ErrorTrait> ErrorTrait for AtomicSaveError<E> {
    fn cause(&self) -> Option<&dyn ErrorTrait> {
        match *self {
            AtomicSaveError::Internal(ref e) => Some(e),
            AtomicSaveError::User(ref e) => Some(e),
        }
    }
}

fn safe_parent(p: &path::Path) -> Option<&path::Path> {
    match p.parent() {
        None => None,
        Some(x) if x.as_os_str().len() == 0 => Some(&path::Path::new(".")),
        x => x,
    }
}

/// Create a file and write to it atomically, in a callback.
pub struct AtomicFile {
    /// Path to the final file that is atomically written.
    path: path::PathBuf,
    overwrite: OverwriteBehavior,
    durability: Durability,
    /// Directory to which to write the temporary subdirectories.
    tmpdir: path::PathBuf,
}

impl AtomicFile {
    /// Helper for writing to the file at `path` atomically, in write-only mode.
    ///
    /// If `OverwriteBehaviour::DisallowOverwrite` is given,
    /// an `Error::Internal` containing an `std::io::ErrorKind::AlreadyExists`
    /// will be returned from `self.write(...)` if the file exists.
    ///
    /// The temporary file is written to a temporary subdirectory in `.`, to ensure
    /// it’s on the same filesystem (so that the move is atomic).
    pub fn new(
        p: &path::Path,
        overwrite: OverwriteBehavior,
        durability: Durability,
    ) -> Self {
        AtomicFile::new_with_tmpdir(
            p,
            overwrite,
            durability,
            safe_parent(p).unwrap_or(path::Path::new(".")),
        )
    }

    /// Like `AtomicFile::new`, but the temporary file is written to a temporary subdirectory in `tmpdir`.
    ///
    /// TODO: does `tmpdir` have to exist?
    pub fn new_with_tmpdir(
        path: &path::Path,
        overwrite: OverwriteBehavior,
        durability: Durability,
        tmpdir: &path::Path,
    ) -> Self {
        AtomicFile {
            path: path.to_path_buf(),
            overwrite,
            durability,
            tmpdir: tmpdir.to_path_buf(),
        }
    }

    /// Move the file to `self.path()`. Not exposed!
    fn commit(self, tmppath: &path::Path) -> io::Result<()> {
        match self.overwrite {
            AllowOverwrite => {
                replace_atomic(tmppath, self.path(), self.durability)
            }
            DisallowOverwrite => {
                move_atomic(tmppath, self.path(), self.durability)
            }
        }
    }

    /// Get the target filepath.
    pub fn path(&self) -> &path::Path {
        &self.path
    }

    /// Open a temporary file, call `f` on it (which is supposed to write to it), then move the
    /// file atomically to `self.path`.
    ///
    /// The temporary file is written to a randomized temporary subdirectory with prefix `.atomicwrite`.
    pub fn write<T, E, F>(self, f: F) -> Result<T, AtomicSaveError<E>>
    where
        F: FnOnce(&mut fs::File) -> Result<T, E>,
    {
        let tmpdir = tempfile::Builder::new()
            .prefix(".atomicwrite")
            .tempdir_in(&self.tmpdir)
            .map_err(AtomicSaveError::Internal)?;

        let tmppath = tmpdir.path().join("tmpfile.tmp");
        let rv = {
            let mut tmpfile = fs::File::create(&tmppath)
                .map_err(AtomicSaveError::Internal)?;
            let r = f(&mut tmpfile).map_err(AtomicSaveError::User)?;
            tmpfile.sync_all().map_err(AtomicSaveError::Internal)?;
            r
        };
        self.commit(&tmppath).map_err(AtomicSaveError::Internal)?;
        Ok(rv)
    }
}

mod imp {
    use super::{safe_parent, Durability};

    use std::os::unix::io::AsRawFd;
    use std::{fs, io, path};

    fn fsync<T: AsRawFd>(f: T) -> io::Result<()> {
        match nix::unistd::fsync(f.as_raw_fd()) {
            Ok(()) => Ok(()),
            Err(nix::Error::Sys(errno)) => Err(errno.into()),
            Err(nix::Error::InvalidPath) => {
                Err(io::Error::new(io::ErrorKind::Other, "invalid path"))
            }
            Err(nix::Error::InvalidUtf8) => {
                Err(io::Error::new(io::ErrorKind::Other, "invalid utf-8"))
            }
            Err(nix::Error::UnsupportedOperation) => Err(io::Error::new(
                io::ErrorKind::Other,
                "unsupported operation",
            )),
        }
    }

    fn fsync_dir(x: &path::Path) -> io::Result<()> {
        let f = fs::File::open(x)?;
        fsync(f)
    }

    /// Move `src` to `dst`. If `dst` exists, it will be silently overwritten.
    ///
    /// Both paths must reside on the same filesystem for the operation to be atomic.
    pub fn replace_atomic(
        src: &path::Path,
        dst: &path::Path,
        durability: Durability,
    ) -> io::Result<()> {
        fs::rename(src, dst)?;

        match durability {
            Durability::SyncDir => {
                let dst_directory = safe_parent(dst).unwrap();
                fsync_dir(dst_directory)?;
            }
            Durability::DontSyncDir => {}
        }

        Ok(())
    }

    /// Move `src` to `dst`. An error will be returned if `dst` exists.
    ///
    /// Both paths must reside on the same filesystem for the operation to be atomic.
    pub fn move_atomic(
        src: &path::Path,
        dst: &path::Path,
        durability: Durability,
    ) -> io::Result<()> {
        fs::hard_link(src, dst)?;
        fs::remove_file(src)?;

        match durability {
            Durability::SyncDir => {
                let src_directory = safe_parent(src).unwrap();
                let dst_directory = safe_parent(dst).unwrap();
                fsync_dir(dst_directory)?;
                if src_directory != dst_directory {
                    fsync_dir(src_directory)?;
                }
            }
            Durability::DontSyncDir => {}
        }

        Ok(())
    }
}

use imp::{move_atomic, replace_atomic};
