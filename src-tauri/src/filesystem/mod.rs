use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::errors::{AppError, AppResult};

pub const PROJECT_JSON_FILE: &str = "project.json";
pub const PROJECT_DB_FILE: &str = "project.db";
pub const ASSETS_DIR: &str = "assets";
pub const BACKUPS_DIR: &str = "backups";
pub const EXPORTS_DIR: &str = "exports";

pub fn ensure_project_directories(project_path: &Path) -> AppResult<()> {
    fs::create_dir_all(project_path)?;
    fs::create_dir_all(project_path.join(ASSETS_DIR))?;
    fs::create_dir_all(project_path.join(BACKUPS_DIR))?;
    fs::create_dir_all(project_path.join(EXPORTS_DIR))?;
    Ok(())
}

pub fn project_json_path(project_path: &Path) -> PathBuf {
    project_path.join(PROJECT_JSON_FILE)
}

pub fn project_db_path(project_path: &Path) -> PathBuf {
    project_path.join(PROJECT_DB_FILE)
}

pub fn write_json_atomic(path: &Path, value: &impl serde::Serialize) -> AppResult<()> {
    let parent = path.parent().ok_or_else(|| {
        AppError::Filesystem(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "atomic write path has no parent",
        ))
    })?;
    let temp_path = path.with_extension("json.tmp");
    let json = serde_json::to_vec_pretty(value)?;

    {
        let mut file = File::create(&temp_path)?;
        file.write_all(&json)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
    }

    fs::rename(&temp_path, path)?;

    if let Ok(parent_dir) = File::open(parent) {
        let _ = parent_dir.sync_all();
    }

    Ok(())
}

pub fn read_json_file<T>(path: &Path) -> AppResult<T>
where
    T: serde::de::DeserializeOwned,
{
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}
