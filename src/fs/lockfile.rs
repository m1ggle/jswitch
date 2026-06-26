use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process,
};

use tracing::{debug, warn};

use crate::error::jswitch_error::IoError;

/// RAII guard that holds an exclusive lockfile.
///
/// On creation it atomically creates a lockfile containing the current process ID.
/// If a stale lockfile exists (owner process no longer running), it is reclaimed.
/// On drop, the lockfile is removed.
#[derive(Debug)]
pub struct Lockfile {
    path: PathBuf,
    /// Whether the lockfile was successfully acquired and should be removed on drop.
    acquired: bool,
}

impl Lockfile {
    /// Attempt to acquire an exclusive lockfile at `path`.
    ///
    /// If a lockfile already exists and its owner process is no longer running,
    /// the stale lockfile is removed and acquisition is retried.
    pub fn acquire(path: &Path) -> Result<Self, IoError> {
        match Self::try_create(path) {
            Ok(()) => Ok(Self {
                path: path.to_path_buf(),
                acquired: true,
            }),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
                if Self::is_stale(path)? {
                    debug!(path = %path.display(), "removing stale lockfile");
                    fs::remove_file(path).map_err(|source| IoError::Access {
                        path: path.to_path_buf(),
                        source,
                    })?;
                    Self::try_create(path).map_err(|source| IoError::Access {
                        path: path.to_path_buf(),
                        source,
                    })?;
                    Ok(Self {
                        path: path.to_path_buf(),
                        acquired: true,
                    })
                } else {
                    Err(IoError::Access {
                        path: path.to_path_buf(),
                        source: io::Error::new(
                            io::ErrorKind::AlreadyExists,
                            "lockfile is held by a running process",
                        ),
                    })
                }
            }
            Err(source) => Err(IoError::Access {
                path: path.to_path_buf(),
                source,
            }),
        }
    }

    /// Manually release the lockfile before drop.
    pub fn release(mut self) -> Result<(), IoError> {
        self.do_release()
    }

    fn do_release(&mut self) -> Result<(), IoError> {
        if !self.acquired {
            return Ok(());
        }
        self.acquired = false;
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                debug!(path = %self.path.display(), "lockfile already gone");
                Ok(())
            }
            Err(source) => Err(IoError::Access {
                path: self.path.clone(),
                source,
            }),
        }
    }

    fn try_create(path: &Path) -> io::Result<()> {
        debug!(path = %path.display(), "acquiring lockfile");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        writeln!(file, "{}", process::id())?;
        Ok(())
    }

    /// Read the PID from a lockfile and check whether that process is still alive.
    fn is_stale(path: &Path) -> Result<bool, IoError> {
        let content = fs::read_to_string(path).map_err(|source| IoError::Access {
            path: path.to_path_buf(),
            source,
        })?;
        let pid: u32 = content.trim().parse().map_err(|_| IoError::Access {
            path: path.to_path_buf(),
            source: io::Error::new(io::ErrorKind::InvalidData, "invalid PID in lockfile"),
        })?;
        Ok(!is_process_alive(pid))
    }
}

impl Drop for Lockfile {
    fn drop(&mut self) {
        if self.acquired
            && let Err(source) = self.do_release()
        {
            warn!(path = %self.path.display(), error = %source, "failed to remove lockfile on drop");
        }
    }
}

/// Check whether a process with the given PID is currently running.
#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    // `kill -0 <pid>` sends no signal — it only checks process existence.
    match process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

/// Check whether a process with the given PID is currently running.
#[cfg(not(unix))]
fn is_process_alive(pid: u32) -> bool {
    // On Windows there is no `kill -0`. Use `tasklist` to check the PID.
    match process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains(&pid.to_string())
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquires_and_releases_lockfile() {
        let dir = tempfile::tempdir().unwrap();
        let lockpath = dir.path().join("jswitch.lock");

        let lock = Lockfile::acquire(&lockpath).unwrap();
        assert!(lockpath.exists());

        lock.release().unwrap();
        assert!(!lockpath.exists());
    }

    #[test]
    fn drop_removes_lockfile() {
        let dir = tempfile::tempdir().unwrap();
        let lockpath = dir.path().join("jswitch.lock");

        {
            let _lock = Lockfile::acquire(&lockpath).unwrap();
            assert!(lockpath.exists());
        }
        assert!(!lockpath.exists());
    }

    #[test]
    fn second_acquire_fails_while_held() {
        let dir = tempfile::tempdir().unwrap();
        let lockpath = dir.path().join("jswitch.lock");

        let _lock1 = Lockfile::acquire(&lockpath).unwrap();
        let result = Lockfile::acquire(&lockpath);
        assert!(result.is_err());
    }

    #[test]
    fn reclaims_stale_lockfile() {
        let dir = tempfile::tempdir().unwrap();
        let lockpath = dir.path().join("jswitch.lock");

        // Write a PID that is essentially guaranteed not to exist.
        fs::write(&lockpath, "999999").unwrap();

        let lock = Lockfile::acquire(&lockpath).unwrap();
        assert!(lockpath.exists());

        lock.release().unwrap();
    }
}
