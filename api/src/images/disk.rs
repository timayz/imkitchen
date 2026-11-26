//! Writing the files a node keeps beside the store.
//!
//! A file is written in full under a temporary name, then takes its own, so
//! a reader never sees half of one. What is written sits in the kernel's
//! cache for a while before the disk has it, and a new file's name may get
//! there first: a power cut in between leaves the name over an empty file,
//! which would be served as a picture a cache keeps for a year. So every
//! file is synced to the disk before it takes its name ([`write_partial`]):
//! a power cut may take a file back, never leave it empty.
//!
//! The name itself is synced too ([`Durability::Synced`]) where something
//! counts on it: a file a majority of the cluster holds before the event
//! naming it commits. A copy a node takes for itself leaves its name to the
//! kernel ([`Durability::Cached`]): its peers hold the file, and a power cut
//! costs a fetch.

use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Whether a write's name is on the disk when it returns, or only its bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    /// The bytes and the name: what a majority of the cluster counts on
    /// before an event names the file.
    Synced,
    /// The bytes, the name following within seconds: a copy a node takes
    /// for itself, which a power cut may take back, to be fetched again.
    Cached,
}

/// Writes `bytes` at `path`: in full under another name first, so the file
/// is whole from the moment it is there. Creates the directory when it is
/// missing.
pub fn write(path: &Path, bytes: &[u8], durability: Durability) -> io::Result<()> {
    let partial = write_partial(path, bytes)?;
    std::fs::rename(&partial, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&partial);
    })?;
    match durability {
        Durability::Synced => sync_dir(parent(path)),
        Durability::Cached => Ok(()),
    }
}

/// Writes `bytes` to a new file next to `path`, named after it with a random
/// part and `.part` added, so no other write takes the same name, and syncs
/// it to the disk. Returns the file's path, for the caller to give it
/// `path`'s name; on an error, it leaves no file behind.
fn write_partial(path: &Path, bytes: &[u8]) -> io::Result<PathBuf> {
    let dir = parent(path);
    create_dir_all(dir)?;
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let partial = dir.join(format!("{name}.{}.part", ulid::Ulid::generate()));
    let written = File::create_new(&partial).and_then(|mut file| {
        file.write_all(bytes)?;
        file.sync_all()
    });
    match written {
        Ok(()) => Ok(partial),
        Err(err) => {
            let _ = std::fs::remove_file(&partial);
            Err(err)
        }
    }
}

/// Syncs the names `dir` holds to the disk: a file that took or lost its
/// name in it keeps that through a power cut.
fn sync_dir(dir: &Path) -> io::Result<()> {
    File::open(dir)?.sync_all()
}

/// Creates `dir` and whatever is missing above it, each directory created
/// synced into its parent, so a file synced into `dir` survives a power cut
/// with the path to it.
fn create_dir_all(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        return Ok(());
    }
    let above = parent(dir);
    create_dir_all(above)?;
    match std::fs::create_dir(dir) {
        Ok(()) => sync_dir(above),
        // Created meanwhile, by another write that syncs it.
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(err) => Err(err),
    }
}

/// The directory holding `path`: `.` for a bare name.
fn parent(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_is_whole_from_the_moment_it_has_its_name() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("a/b/one.webp");

        let partial = write_partial(&path, b"first").unwrap();
        assert_eq!(partial.parent(), path.parent());
        assert_eq!(std::fs::read(&partial).unwrap(), b"first");
        assert!(!path.exists(), "the name is given once the bytes are there");
        std::fs::remove_file(partial).unwrap();

        write(&path, b"first", Durability::Synced).unwrap();
        write(&path, b"other", Durability::Cached).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"other");
        let left: Vec<_> = std::fs::read_dir(path.parent().unwrap()).unwrap().collect();
        assert_eq!(left.len(), 1, "nothing is left of the temporary names");
    }
}
