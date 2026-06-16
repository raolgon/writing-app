use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("project not found: {0}")]
    ProjectNotFound(String),
    #[error("unsupported project version: {0}")]
    UnsupportedProjectVersion(u32),
    #[error("project is locked: {0}")]
    ProjectLocked(String),
    #[error("project is corrupt: {0}")]
    ProjectCorrupt(String),
    #[error("migration failed: {0}")]
    MigrationFailed(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("filesystem error: {0}")]
    Filesystem(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("project session not found: {0}")]
    SessionNotFound(String),
    #[error("document revision conflict: expected {expected}, found {actual}")]
    RevisionConflict { expected: i64, actual: i64 },
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(error: AppError) -> Self {
        let code = match &error {
            AppError::ProjectNotFound(_) => "ProjectNotFound",
            AppError::UnsupportedProjectVersion(_) => "UnsupportedProjectVersion",
            AppError::ProjectLocked(_) => "ProjectLocked",
            AppError::ProjectCorrupt(_) => "ProjectCorrupt",
            AppError::MigrationFailed(_) => "MigrationFailed",
            AppError::Validation(_) => "ValidationError",
            AppError::Database(_) => "DatabaseError",
            AppError::Filesystem(_) => "FilesystemError",
            AppError::Json(_) => "ProjectCorrupt",
            AppError::SessionNotFound(_) => "SessionNotFound",
            AppError::RevisionConflict { .. } => "RevisionConflict",
        };

        Self {
            code,
            message: error.to_string(),
        }
    }
}
