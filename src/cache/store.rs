//! The cache folder: private permissions, no symbolic links, atomic writes.

// Stubs until the implementation commit.
#![allow(dead_code, unused_imports)]

use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, DirBuilder, File, OpenOptions, Permissions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

const DIR_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;
/// Group and other bits: any of them set means the entry is too open.
const NOT_PRIVATE: u32 = 0o077;
const FOLDER_NAME: &str = "thedailybugle";

#[derive(Debug)]
pub enum CacheError {
    /// The folder is a symbolic link or not a folder.
    NotAPrivateFolder,
    Io(io::Error),
}

impl fmt::Display for CacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAPrivateFolder => {
                formatter.write_str("cache path is a symbolic link or not a folder")
            }
            Self::Io(error) => write!(formatter, "cache I/O failed: {}", error.kind()),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<io::Error> for CacheError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// `$XDG_CACHE_HOME/thedailybugle`, or `$HOME/.cache/thedailybugle`. Relative
/// values are ignored, as the XDG spec says.
pub fn default_cache_path(
    _xdg_cache_home: Option<&OsStr>,
    _home: Option<&OsStr>,
) -> Option<PathBuf> {
    todo!()
}

#[derive(Debug, Clone)]
pub struct CacheDir {
    path: PathBuf,
}

impl CacheDir {
    /// Opens the folder, creating it `0700` if needed and tightening it if it
    /// is more open. A symbolic link is refused, wherever it points.
    pub fn open(_path: &Path) -> Result<Self, CacheError> {
        todo!()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The file's bytes, or `None` when it is missing, not a regular file (a
    /// symbolic link included), or larger than `max_bytes`.
    pub fn read(&self, _name: &str, _max_bytes: usize) -> Option<Vec<u8>> {
        todo!()
    }

    /// Writes a new file next to the old one and renames it over, so readers
    /// see either the old bytes or the new ones. `rename` replaces a symbolic
    /// link itself instead of writing through it.
    pub fn write_atomic(&self, _name: &str, _bytes: &[u8]) -> Result<(), CacheError> {
        todo!()
    }
}

fn write_new_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    // A leftover temporary file from a crashed run is replaced, not appended.
    let _ = fs::remove_file(path);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(FILE_MODE)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mode(path: &Path) -> u32 {
        fs::symlink_metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn default_path_prefers_an_absolute_xdg_cache_home() {
        assert_eq!(
            default_cache_path(Some(OsStr::new("/x/cache")), Some(OsStr::new("/home/u"))),
            Some(PathBuf::from("/x/cache/thedailybugle"))
        );
        assert_eq!(
            default_cache_path(Some(OsStr::new("relative")), Some(OsStr::new("/home/u"))),
            Some(PathBuf::from("/home/u/.cache/thedailybugle"))
        );
        assert_eq!(
            default_cache_path(None, Some(OsStr::new("/home/u"))),
            Some(PathBuf::from("/home/u/.cache/thedailybugle"))
        );
        assert_eq!(default_cache_path(None, Some(OsStr::new("relative"))), None);
        assert_eq!(default_cache_path(None, None), None);
    }

    #[test]
    fn new_folder_and_files_are_private() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("nested").join("thedailybugle");

        let cache = CacheDir::open(&path).unwrap();
        cache.write_atomic("kev.json", b"{}").unwrap();

        assert_eq!(mode(&path), 0o700);
        assert_eq!(mode(&path.join("kev.json")), 0o600);
        assert_eq!(cache.read("kev.json", 10).unwrap(), b"{}");
    }

    #[test]
    fn open_folder_and_files_are_tightened() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("thedailybugle");
        fs::create_dir(&path).unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o755)).unwrap();
        fs::write(path.join("kev.json"), b"{}").unwrap();
        fs::set_permissions(path.join("kev.json"), Permissions::from_mode(0o644)).unwrap();

        let cache = CacheDir::open(&path).unwrap();
        assert!(cache.read("kev.json", 10).is_some());

        assert_eq!(mode(&path), 0o700);
        assert_eq!(mode(&path.join("kev.json")), 0o600);
    }

    #[test]
    fn folder_that_is_a_symbolic_link_is_refused() {
        let root = tempfile::tempdir().unwrap();
        let elsewhere = root.path().join("elsewhere");
        fs::create_dir(&elsewhere).unwrap();
        let link = root.path().join("thedailybugle");
        std::os::unix::fs::symlink(&elsewhere, &link).unwrap();

        assert!(matches!(
            CacheDir::open(&link),
            Err(CacheError::NotAPrivateFolder)
        ));
        let file = root.path().join("file");
        fs::write(&file, b"").unwrap();
        assert!(matches!(
            CacheDir::open(&file),
            Err(CacheError::NotAPrivateFolder)
        ));
    }

    #[test]
    fn file_that_is_a_symbolic_link_is_not_read_and_not_written_through() {
        let root = tempfile::tempdir().unwrap();
        let secret = root.path().join("id_ed25519");
        fs::write(&secret, b"PRIVATE KEY").unwrap();
        let cache = CacheDir::open(&root.path().join("thedailybugle")).unwrap();
        std::os::unix::fs::symlink(&secret, cache.path().join("kev.json")).unwrap();

        assert_eq!(cache.read("kev.json", 1024), None);
        cache.write_atomic("kev.json", b"new").unwrap();

        assert_eq!(fs::read(&secret).unwrap(), b"PRIVATE KEY");
        assert!(
            !fs::symlink_metadata(cache.path().join("kev.json"))
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn file_over_the_limit_is_not_read() {
        let root = tempfile::tempdir().unwrap();
        let cache = CacheDir::open(&root.path().join("thedailybugle")).unwrap();
        cache.write_atomic("kev.json", &[b'x'; 11]).unwrap();

        assert_eq!(cache.read("kev.json", 10), None);
        assert_eq!(cache.read("kev.json", 11).unwrap().len(), 11);
        assert_eq!(cache.read("missing.json", 10), None);
    }

    #[test]
    fn failed_write_leaves_the_old_file_intact() {
        let root = tempfile::tempdir().unwrap();
        let cache = CacheDir::open(&root.path().join("thedailybugle")).unwrap();
        cache.write_atomic("kev.json", b"old").unwrap();
        // A read-only folder: the temporary file cannot be created.
        fs::set_permissions(cache.path(), Permissions::from_mode(0o500)).unwrap();

        let result = cache.write_atomic("kev.json", b"new");

        fs::set_permissions(cache.path(), Permissions::from_mode(0o700)).unwrap();
        assert!(matches!(result, Err(CacheError::Io(_))));
        assert_eq!(cache.read("kev.json", 10).unwrap(), b"old");
        assert_eq!(fs::read_dir(cache.path()).unwrap().count(), 1);
    }

    #[test]
    fn cache_errors_explain_themselves() {
        assert_eq!(
            CacheError::NotAPrivateFolder.to_string(),
            "cache path is a symbolic link or not a folder"
        );
        let error = CacheError::from(io::Error::from(io::ErrorKind::PermissionDenied));
        assert_eq!(error.to_string(), "cache I/O failed: permission denied");
    }
}
