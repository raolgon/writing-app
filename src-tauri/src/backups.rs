use std::fs;
use std::path::Path;

use rusqlite::{params, Connection};
use serde_json::json;
use uuid::Uuid;

use crate::database::{open_project_database, CURRENT_SCHEMA_VERSION};
use crate::domain::{BackupKind, BackupRecord, ProjectJson, PROJECT_FORMAT_VERSION};
use crate::errors::{AppError, AppResult};
use crate::filesystem::{
    project_db_path, project_json_path, read_json_file, write_json_atomic, BACKUPS_DIR,
};
use crate::time::now_iso;

const DEFAULT_RETENTION_COUNT: usize = 10;

struct BackupRecordRow {
    id: Uuid,
    created_at: String,
    path: String,
    kind: String,
    format_version: u32,
    size_bytes: Option<i64>,
    status: String,
}

pub fn create_manual_backup(project_path: &Path) -> AppResult<BackupRecord> {
    create_backup(project_path, BackupKind::Manual)
}

pub fn create_automatic_backup(project_path: &Path) -> AppResult<BackupRecord> {
    create_backup(project_path, BackupKind::Automatic)
}

pub fn list_backups(project_path: &Path) -> AppResult<Vec<BackupRecord>> {
    let conn = open_project_database(&project_db_path(project_path))?;
    backup_records(&conn)
}

fn create_backup(project_path: &Path, kind: BackupKind) -> AppResult<BackupRecord> {
    let project_json: ProjectJson = read_json_file(&project_json_path(project_path))?;
    let retention_count = backup_retention_count(&project_json);
    let conn = open_project_database(&project_db_path(project_path))?;
    checkpoint_database(&conn)?;

    let id = Uuid::new_v4();
    let created_at = now_iso();
    let relative_path = backup_relative_path(&created_at, id);
    let backups_dir = project_path.join(BACKUPS_DIR);
    fs::create_dir_all(&backups_dir)?;
    let final_dir = project_path.join(&relative_path);
    let temp_dir = backups_dir.join(format!(".{id}.tmp"));

    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir_all(&temp_dir)?;

    let result = (|| -> AppResult<i64> {
        copy_project_files(project_path, &temp_dir)?;
        let manifest = json!({
            "id": id,
            "createdAt": created_at,
            "kind": kind,
            "formatVersion": PROJECT_FORMAT_VERSION,
            "databaseSchemaVersion": CURRENT_SCHEMA_VERSION,
            "appVersion": env!("CARGO_PKG_VERSION"),
        });
        write_json_atomic(&temp_dir.join("backup.json"), &manifest)?;
        let size_bytes = directory_size(&temp_dir)?;
        fs::rename(&temp_dir, &final_dir)?;
        Ok(size_bytes)
    })();

    match result {
        Ok(size_bytes) => {
            let record = insert_backup_record(
                &conn,
                id,
                &created_at,
                &relative_path,
                kind,
                Some(size_bytes),
                "complete",
            )?;
            prune_backups(project_path, &conn, retention_count)?;
            Ok(record)
        }
        Err(error) => {
            let _ = fs::remove_dir_all(&temp_dir);
            let _ =
                insert_backup_record(&conn, id, &created_at, &relative_path, kind, None, "failed");
            Err(error)
        }
    }
}

fn checkpoint_database(conn: &Connection) -> AppResult<()> {
    conn.pragma_update(None, "wal_checkpoint", "TRUNCATE")?;
    Ok(())
}

fn copy_project_files(project_path: &Path, backup_dir: &Path) -> AppResult<()> {
    fs::copy(
        project_json_path(project_path),
        backup_dir.join("project.json"),
    )?;
    fs::copy(project_db_path(project_path), backup_dir.join("project.db"))?;

    let assets_dir = project_path.join("assets");
    if assets_dir.exists() {
        copy_dir_recursive(&assets_dir, &backup_dir.join("assets"))?;
    }

    Ok(())
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> AppResult<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn directory_size(path: &Path) -> AppResult<i64> {
    let mut size = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            size += directory_size(&entry.path())?;
        } else {
            size += metadata.len() as i64;
        }
    }
    Ok(size)
}

fn insert_backup_record(
    conn: &Connection,
    id: Uuid,
    created_at: &str,
    path: &str,
    kind: BackupKind,
    size_bytes: Option<i64>,
    status: &str,
) -> AppResult<BackupRecord> {
    conn.execute(
        "INSERT INTO backup_records (
            id, created_at, path, kind, format_version, size_bytes, status
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id.to_string(),
            created_at,
            path,
            kind.as_str(),
            PROJECT_FORMAT_VERSION,
            size_bytes,
            status,
        ],
    )?;

    Ok(BackupRecord {
        id,
        created_at: created_at.to_string(),
        path: path.to_string(),
        kind,
        format_version: PROJECT_FORMAT_VERSION,
        size_bytes,
        status: status.to_string(),
    })
}

fn backup_records(conn: &Connection) -> AppResult<Vec<BackupRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, created_at, path, kind, format_version, size_bytes, status
         FROM backup_records
         ORDER BY created_at DESC, path DESC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(BackupRecordRow {
                id: parse_uuid(row.get(0)?)?,
                created_at: row.get(1)?,
                path: row.get(2)?,
                kind: row.get(3)?,
                format_version: row.get::<_, i64>(4)? as u32,
                size_bytes: row.get(5)?,
                status: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    rows.into_iter()
        .map(|row| {
            Ok(BackupRecord {
                id: row.id,
                created_at: row.created_at,
                path: row.path,
                kind: BackupKind::try_from(row.kind.as_str()).map_err(AppError::ProjectCorrupt)?,
                format_version: row.format_version,
                size_bytes: row.size_bytes,
                status: row.status,
            })
        })
        .collect()
}

fn prune_backups(project_path: &Path, conn: &Connection, retention_count: usize) -> AppResult<()> {
    let records = backup_records(conn)?
        .into_iter()
        .filter(|record| record.status == "complete")
        .collect::<Vec<_>>();

    for record in records.into_iter().skip(retention_count) {
        let path = project_path.join(&record.path);
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        conn.execute(
            "DELETE FROM backup_records WHERE id = ?1",
            params![record.id.to_string()],
        )?;
    }

    Ok(())
}

fn backup_retention_count(project: &ProjectJson) -> usize {
    project
        .settings
        .get("backupRetentionCount")
        .and_then(|value| value.as_u64())
        .map(|value| value.max(1) as usize)
        .unwrap_or(DEFAULT_RETENTION_COUNT)
}

fn backup_relative_path(created_at: &str, id: Uuid) -> String {
    let timestamp = created_at
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>();
    format!("{BACKUPS_DIR}/{timestamp}-{id}")
}

fn parse_uuid(value: String) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(&value).map_err(to_sql_error)
}

fn to_sql_error<E>(error: E) -> rusqlite::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    rusqlite::Error::ToSqlConversionFailure(Box::new(error))
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use serde_json::json;
    use tempfile::tempdir;

    use super::{create_manual_backup, list_backups};
    use crate::domain::{CreateProjectRequest, ProjectJson, ProjectType};
    use crate::filesystem::{project_json_path, read_json_file, write_json_atomic};
    use crate::project;

    #[test]
    fn creates_backup_directory_and_record() {
        let dir = tempdir().expect("tempdir");
        let project_dir = dir.path().join("draft");
        project::create_project(&CreateProjectRequest {
            folder_path: project_dir.display().to_string(),
            title: "Draft".to_string(),
            description: String::new(),
            project_type: ProjectType::Blank,
        })
        .expect("create project");

        let record = create_manual_backup(&project_dir).expect("backup");

        assert_eq!(record.kind.as_str(), "manual");
        assert!(project_dir.join(&record.path).join("project.json").exists());
        assert!(project_dir.join(&record.path).join("project.db").exists());
        assert!(project_dir.join(&record.path).join("backup.json").exists());
        assert_eq!(list_backups(&project_dir).expect("records").len(), 1);
    }

    #[test]
    fn applies_backup_retention() {
        let dir = tempdir().expect("tempdir");
        let project_dir = dir.path().join("draft");
        project::create_project(&CreateProjectRequest {
            folder_path: project_dir.display().to_string(),
            title: "Draft".to_string(),
            description: String::new(),
            project_type: ProjectType::Blank,
        })
        .expect("create project");

        let json_path = project_json_path(&project_dir);
        let mut project_json: ProjectJson = read_json_file(&json_path).expect("project json");
        project_json.settings["backupRetentionCount"] = json!(2);
        write_json_atomic(&json_path, &project_json).expect("write settings");

        for _ in 0..4 {
            create_manual_backup(&project_dir).expect("backup");
        }

        let records = list_backups(&project_dir).expect("records");
        assert_eq!(
            records
                .iter()
                .filter(|record| record.status == "complete")
                .count(),
            2
        );

        let conn = Connection::open(project_dir.join("project.db")).expect("open db");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM backup_records", [], |row| row.get(0))
            .expect("count");
        assert_eq!(count, 2);
    }
}
