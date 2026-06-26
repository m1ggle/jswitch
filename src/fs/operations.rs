use std::{
    fs, io,
    path::{Path, PathBuf},
};

use tracing::debug;

/// Create a directory and all of its parent components if they do not exist.
pub fn create_dir_all(path: &Path) -> io::Result<()> {
    debug!(path = %path.display(), "creating directory tree");
    fs::create_dir_all(path)
}

/// Remove a directory and all of its contents recursively.
pub fn remove_dir(path: &Path) -> io::Result<()> {
    debug!(path = %path.display(), "removing directory tree");
    fs::remove_dir_all(path)
}

/// Remove a single file.
pub fn remove_file(path: &Path) -> io::Result<()> {
    debug!(path = %path.display(), "removing file");
    fs::remove_file(path)
}

/// Rename or move a file or directory to a new path.
pub fn rename(from: &Path, to: &Path) -> io::Result<()> {
    debug!(from = %from.display(), to = %to.display(), "renaming");
    fs::rename(from, to)
}

/// Write text content to a file, creating it if it does not exist.
pub fn write_file(path: &Path, content: &str) -> io::Result<()> {
    debug!(path = %path.display(), "writing file");
    fs::write(path, content)
}

/// Read the entire contents of a file as a UTF-8 string.
pub fn read_file(path: &Path) -> io::Result<String> {
    debug!(path = %path.display(), "reading file");
    fs::read_to_string(path)
}

/// Collect all entries in a directory as `PathBuf` values.
///
/// Returns an empty vector if the directory does not exist.
pub fn read_dir_entries(path: &Path) -> io::Result<Vec<PathBuf>> {
    debug!(path = %path.display(), "reading directory entries");
    fs::read_dir(path)?
        .map(|entry| entry.map(|e| e.path()))
        .collect()
}

/// Check whether a path points to an existing directory.
pub fn dir_exists(path: &Path) -> bool {
    path.is_dir()
}

/// Check whether a path points to an existing regular file.
pub fn file_exists(path: &Path) -> bool {
    path.is_file()
}

/// Remove an existing directory tree and recreate it as an empty directory.
///
/// If the path does not exist, it is simply created.
pub fn reset_dir(path: &Path) -> io::Result<()> {
    debug!(path = %path.display(), "resetting directory");
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_removes_directory() {
        let dir = tempfile::tempdir().unwrap();
        let new_dir = dir.path().join("nested/deep");

        create_dir_all(&new_dir).unwrap();
        assert!(dir_exists(&new_dir));

        remove_dir(&new_dir).unwrap();
        assert!(!dir_exists(&new_dir));
    }

    #[test]
    fn writes_and_reads_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        write_file(&file_path, "hello world").unwrap();
        assert!(file_exists(&file_path));

        let content = read_file(&file_path).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn renames_file() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("from.txt");
        let to = dir.path().join("to.txt");

        write_file(&from, "data").unwrap();
        rename(&from, &to).unwrap();

        assert!(!file_exists(&from));
        assert!(file_exists(&to));
    }

    #[test]
    fn reads_dir_entries_returns_paths() {
        let dir = tempfile::tempdir().unwrap();
        write_file(&dir.path().join("a"), "").unwrap();
        write_file(&dir.path().join("b"), "").unwrap();

        let entries = read_dir_entries(dir.path()).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn reset_dir_replaces_existing_content() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        create_dir_all(&target).unwrap();
        write_file(&target.join("old"), "old").unwrap();

        reset_dir(&target).unwrap();

        assert!(dir_exists(&target));
        assert!(!file_exists(&target.join("old")));
    }

    #[test]
    fn read_dir_entries_returns_empty_for_missing_dir() {
        let result = read_dir_entries(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
