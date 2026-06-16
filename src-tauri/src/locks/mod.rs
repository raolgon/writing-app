use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::domain::LockFile;
use crate::errors::{AppError, AppResult};
use crate::time::now_iso;

const LOCK_FILE: &str = ".write-lock";

#[derive(Debug)]
pub struct ProjectLock {
    path: PathBuf,
}

impl ProjectLock {
    pub fn acquire(project_path: &Path) -> AppResult<Self> {
        let path = project_path.join(LOCK_FILE);
        let lock = LockFile {
            app: crate::domain::APP_NAME.to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            pid: std::process::id(),
            host: std::env::var("HOSTNAME").ok(),
            created_at: now_iso(),
            heartbeat_at: now_iso(),
        };
        let json = serde_json::to_vec_pretty(&lock)?;

        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(&json)?;
                file.write_all(b"\n")?;
                file.sync_all()?;
                Ok(Self { path })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(AppError::ProjectLocked(path.display().to_string()))
            }
            Err(error) => Err(AppError::Filesystem(error)),
        }
    }

    pub fn release(self) -> AppResult<()> {
        let path = self.path.clone();
        drop(self);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}

impl Drop for ProjectLock {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::ProjectLock;

    #[test]
    fn prevents_duplicate_writer_lock() {
        let dir = tempdir().expect("tempdir");
        let _lock = ProjectLock::acquire(dir.path()).expect("first lock");

        let second = ProjectLock::acquire(dir.path());

        assert!(second.is_err());
    }
}
