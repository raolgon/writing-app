use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};
use serde_json::json;
use uuid::Uuid;

use crate::database::{open_project_database, CURRENT_SCHEMA_VERSION};
use crate::domain::{
    CreateDefaultProjectRequest, CreateProjectRequest, ProjectDatabaseJson, ProjectJson,
    ProjectSummary, ProjectType, PROJECT_FORMAT, PROJECT_FORMAT_VERSION,
};
use crate::errors::{AppError, AppResult};
use crate::filesystem::{
    ensure_project_directories, project_db_path, project_json_path, read_json_file,
    write_json_atomic,
};
use crate::time::now_iso;

pub fn create_project(request: &CreateProjectRequest) -> AppResult<ProjectSummary> {
    validate_project_title(&request.title)?;
    let project_path = normalize_project_path(&request.folder_path)?;
    ensure_new_or_empty_project_folder(&project_path)?;
    ensure_project_directories(&project_path)?;

    let now = now_iso();
    let mut project_json = ProjectJson {
        format: PROJECT_FORMAT.to_string(),
        format_version: PROJECT_FORMAT_VERSION,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        id: Uuid::new_v4(),
        title: request.title.trim().to_string(),
        description: request.description.trim().to_string(),
        project_type: request.project_type,
        created_at: now.clone(),
        updated_at: now.clone(),
        last_opened_at: Some(now.clone()),
        settings: json!({
            "backupRetentionCount": 10,
            "editor": {
                "textWidth": "medium",
                "fontSize": 18,
                "lineHeight": 1.7
            }
        }),
        custom_metadata_schema: Vec::new(),
        database: ProjectDatabaseJson {
            file: "project.db".to_string(),
            schema_version: CURRENT_SCHEMA_VERSION,
        },
    };

    write_json_atomic(&project_json_path(&project_path), &project_json)?;
    let mut conn = open_project_database(&project_db_path(&project_path))?;
    insert_project(&mut conn, &project_json)?;
    insert_template_items(&mut conn, project_json.id, request.project_type, &now)?;
    write_session_state(&conn, "lastOpenedAt", &json!(now))?;

    project_json.database.schema_version = CURRENT_SCHEMA_VERSION;
    write_json_atomic(&project_json_path(&project_path), &project_json)?;

    Ok(summary_from_json(project_json, &project_path))
}

pub fn create_default_project(request: &CreateDefaultProjectRequest) -> AppResult<ProjectSummary> {
    validate_project_title(&request.title)?;
    let base_dir = default_projects_base_dir()?;
    std::fs::create_dir_all(&base_dir)?;
    let project_path = unique_project_path(&base_dir, &request.title);

    create_project(&CreateProjectRequest {
        folder_path: project_path.display().to_string(),
        title: request.title.clone(),
        description: request.description.clone(),
        project_type: request.project_type,
    })
}

pub fn open_project(project_path: &Path) -> AppResult<ProjectSummary> {
    let project_json_path = project_json_path(project_path);
    if !project_json_path.exists() {
        return Err(AppError::ProjectNotFound(
            project_path.display().to_string(),
        ));
    }

    let mut project_json: ProjectJson = read_json_file(&project_json_path)?;
    validate_project_json(&project_json)?;

    let now = now_iso();
    let conn = open_project_database(&project_db_path(project_path))?;
    conn.execute(
        "UPDATE projects SET last_opened_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![now, now, project_json.id.to_string()],
    )?;
    write_session_state(&conn, "lastOpenedAt", &json!(now))?;

    project_json.last_opened_at = Some(now);
    project_json.database.schema_version = CURRENT_SCHEMA_VERSION;
    write_json_atomic(&project_json_path, &project_json)?;

    Ok(summary_from_json(project_json, project_path))
}

pub fn validate_project_json(project: &ProjectJson) -> AppResult<()> {
    if project.format != PROJECT_FORMAT {
        return Err(AppError::ProjectCorrupt(format!(
            "expected format {PROJECT_FORMAT}, found {}",
            project.format
        )));
    }

    if project.format_version != PROJECT_FORMAT_VERSION {
        return Err(AppError::UnsupportedProjectVersion(project.format_version));
    }

    if project.database.file != "project.db" {
        return Err(AppError::ProjectCorrupt(format!(
            "unexpected database file {}",
            project.database.file
        )));
    }

    Ok(())
}

fn validate_project_title(title: &str) -> AppResult<()> {
    if title.trim().is_empty() {
        return Err(AppError::Validation(
            "project title cannot be empty".to_string(),
        ));
    }

    Ok(())
}

fn normalize_project_path(folder_path: &str) -> AppResult<PathBuf> {
    if folder_path.trim().is_empty() {
        return Err(AppError::Validation(
            "project folder path cannot be empty".to_string(),
        ));
    }

    Ok(PathBuf::from(folder_path))
}

fn default_projects_base_dir() -> AppResult<PathBuf> {
    if let Ok(path) = std::env::var("LOCAL_WRITER_PROJECTS_DIR") {
        if !path.trim().is_empty() {
            return Ok(PathBuf::from(path));
        }
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| AppError::Validation("could not determine user home directory".to_string()))?;
    let home_path = PathBuf::from(home);
    let documents = home_path.join("Documents");
    let base = if documents.exists() {
        documents
    } else {
        home_path
    };

    Ok(base.join("Local Writer Projects"))
}

fn unique_project_path(base_dir: &Path, title: &str) -> PathBuf {
    let slug = sanitize_project_folder_name(title);
    let first = base_dir.join(&slug);
    if !first.exists() {
        return first;
    }

    for index in 2.. {
        let candidate = base_dir.join(format!("{slug}-{index}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("unbounded loop should return a unique project path")
}

fn sanitize_project_folder_name(title: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for character in title.trim().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "untitled-project".to_string()
    } else {
        slug
    }
}

fn ensure_new_or_empty_project_folder(project_path: &Path) -> AppResult<()> {
    let project_json = project_json_path(project_path);
    let project_db = project_db_path(project_path);

    if project_json.exists() || project_db.exists() {
        return Err(AppError::Validation(format!(
            "project folder already contains {} or {}",
            crate::filesystem::PROJECT_JSON_FILE,
            crate::filesystem::PROJECT_DB_FILE
        )));
    }

    Ok(())
}

fn insert_project(conn: &mut Connection, project: &ProjectJson) -> AppResult<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO projects (
            id, title, description, project_type, created_at, updated_at, last_opened_at,
            settings_json, custom_metadata_schema_json, format_version
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            project.id.to_string(),
            project.title,
            project.description,
            project.project_type.as_str(),
            project.created_at,
            project.updated_at,
            project.last_opened_at,
            serde_json::to_string(&project.settings)?,
            serde_json::to_string(&project.custom_metadata_schema)?,
            project.format_version,
        ],
    )?;
    tx.commit()?;
    Ok(())
}

fn insert_template_items(
    conn: &mut Connection,
    project_id: Uuid,
    project_type: ProjectType,
    now: &str,
) -> AppResult<()> {
    let titles: &[(&str, &str)] = match project_type {
        ProjectType::Blank => &[],
        ProjectType::Novel => &[
            ("Manuscript", "folder"),
            ("Characters", "folder"),
            ("Locations", "folder"),
            ("Research", "research"),
            ("Notes", "note"),
        ],
        ProjectType::Screenplay => &[
            ("Script", "folder"),
            ("Characters", "folder"),
            ("Locations", "folder"),
            ("Research", "research"),
            ("Notes", "note"),
        ],
    };

    let tx = conn.transaction()?;
    for (position, (title, item_type)) in titles.iter().enumerate() {
        tx.execute(
            "INSERT INTO binder_items (
                id, project_id, parent_id, type, title, synopsis, position,
                created_at, updated_at, is_expanded, is_archived
            ) VALUES (?1, ?2, NULL, ?3, ?4, '', ?5, ?6, ?7, 1, 0)",
            params![
                Uuid::new_v4().to_string(),
                project_id.to_string(),
                item_type,
                title,
                position as i64,
                now,
                now,
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

fn write_session_state(conn: &Connection, key: &str, value: &serde_json::Value) -> AppResult<()> {
    conn.execute(
        "INSERT INTO session_state (key, value_json, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at",
        params![key, serde_json::to_string(value)?, now_iso()],
    )?;
    Ok(())
}

fn summary_from_json(project: ProjectJson, project_path: &Path) -> ProjectSummary {
    ProjectSummary {
        id: project.id,
        title: project.title,
        description: project.description,
        project_type: project.project_type,
        path: project_path.display().to_string(),
        format_version: project.format_version,
        database_schema_version: project.database.schema_version,
        last_opened_at: project.last_opened_at,
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{create_project, open_project};
    use crate::domain::{CreateProjectRequest, ProjectType};
    use crate::filesystem::{project_db_path, project_json_path};

    #[test]
    fn creates_project_folder_format() {
        let dir = tempdir().expect("tempdir");
        let project_dir = dir.path().join("draft");

        let summary = create_project(&CreateProjectRequest {
            folder_path: project_dir.display().to_string(),
            title: "Draft".to_string(),
            description: String::new(),
            project_type: ProjectType::Novel,
        })
        .expect("create project");

        assert_eq!(summary.title, "Draft");
        assert!(project_json_path(&project_dir).exists());
        assert!(project_db_path(&project_dir).exists());
        assert!(project_dir.join("assets").exists());
        assert!(project_dir.join("backups").exists());
        assert!(project_dir.join("exports").exists());
    }

    #[test]
    fn opens_existing_project() {
        let dir = tempdir().expect("tempdir");
        let project_dir = dir.path().join("draft");
        create_project(&CreateProjectRequest {
            folder_path: project_dir.display().to_string(),
            title: "Draft".to_string(),
            description: String::new(),
            project_type: ProjectType::Blank,
        })
        .expect("create project");

        let summary = open_project(&project_dir).expect("open project");

        assert_eq!(summary.title, "Draft");
        assert_eq!(summary.format_version, 1);
    }

    #[test]
    fn sanitizes_default_project_folder_names() {
        assert_eq!(
            super::sanitize_project_folder_name("Mi Novela!"),
            "mi-novela"
        );
        assert_eq!(
            super::sanitize_project_folder_name("   "),
            "untitled-project"
        );
    }

    #[test]
    fn chooses_unique_default_project_path() {
        let dir = tempdir().expect("tempdir");
        let first = dir.path().join("draft");
        std::fs::create_dir_all(&first).expect("existing folder");

        let selected = super::unique_project_path(dir.path(), "Draft");

        assert_eq!(selected, dir.path().join("draft-2"));
    }
}
