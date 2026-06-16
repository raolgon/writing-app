use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::database::open_project_database;
use crate::domain::{
    ClearDocumentRecoveryRequest, DocumentRecord, DocumentRecoveryState,
    GetDocumentRecoveryRequest, GetDocumentRequest, RecordDocumentRecoveryRequest,
    SaveDocumentRequest,
};
use crate::errors::{AppError, AppResult};
use crate::filesystem::project_db_path;
use crate::time::now_iso;

const EMPTY_TIPTAP_DOCUMENT: &str =
    r#"{"type":"doc","content":[{"type":"paragraph","content":[]}]}"#;

pub fn get(project_path: &Path, request: &GetDocumentRequest) -> AppResult<DocumentRecord> {
    let mut conn = open_project_database(&project_db_path(project_path))?;
    ensure_document_for_binder_item(&mut conn, request.binder_item_id)?;
    get_by_binder_item_id(&conn, request.binder_item_id)
}

pub fn save(project_path: &Path, request: &SaveDocumentRequest) -> AppResult<DocumentRecord> {
    let mut conn = open_project_database(&project_db_path(project_path))?;
    ensure_document_for_binder_item(&mut conn, request.binder_item_id)?;

    let current = get_by_binder_item_id(&conn, request.binder_item_id)?;
    if let Some(expected_revision) = request.expected_revision {
        if current.revision != expected_revision {
            return Err(AppError::RevisionConflict {
                expected: expected_revision,
                actual: current.revision,
            });
        }
    }

    let now = now_iso();
    let word_count = count_words(&request.content_plain_text);
    let character_count = request.content_plain_text.chars().count() as i64;
    let content_json = serde_json::to_string(&request.content_json)?;
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE documents
         SET content_json = ?1,
             content_plain_text = ?2,
             word_count = ?3,
             character_count = ?4,
             revision = revision + 1,
             updated_at = ?5
         WHERE binder_item_id = ?6",
        params![
            content_json,
            request.content_plain_text,
            word_count,
            character_count,
            now,
            request.binder_item_id.to_string(),
        ],
    )?;
    tx.execute(
        "UPDATE binder_items SET updated_at = ?1 WHERE id = ?2",
        params![now, request.binder_item_id.to_string()],
    )?;
    tx.execute(
        "DELETE FROM session_state WHERE key = ?1",
        params![recovery_key(request.binder_item_id)],
    )?;
    tx.commit()?;

    get_by_binder_item_id(&conn, request.binder_item_id)
}

pub fn record_recovery(
    project_path: &Path,
    request: &RecordDocumentRecoveryRequest,
) -> AppResult<DocumentRecoveryState> {
    let conn = open_project_database(&project_db_path(project_path))?;
    let now = now_iso();
    let recovery = DocumentRecoveryState {
        binder_item_id: request.binder_item_id,
        content_json: request.content_json.clone(),
        content_plain_text: request.content_plain_text.clone(),
        revision: request.revision,
        updated_at: now.clone(),
    };
    conn.execute(
        "INSERT INTO session_state (key, value_json, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at",
        params![
            recovery_key(request.binder_item_id),
            serde_json::to_string(&recovery)?,
            now,
        ],
    )?;
    Ok(recovery)
}

pub fn get_recovery(
    project_path: &Path,
    request: &GetDocumentRecoveryRequest,
) -> AppResult<Option<DocumentRecoveryState>> {
    let conn = open_project_database(&project_db_path(project_path))?;
    let value: Option<String> = conn
        .query_row(
            "SELECT value_json FROM session_state WHERE key = ?1",
            params![recovery_key(request.binder_item_id)],
            |row| row.get(0),
        )
        .optional()?;
    value
        .map(|json| serde_json::from_str(&json).map_err(AppError::from))
        .transpose()
}

pub fn clear_recovery(
    project_path: &Path,
    request: &ClearDocumentRecoveryRequest,
) -> AppResult<()> {
    let conn = open_project_database(&project_db_path(project_path))?;
    conn.execute(
        "DELETE FROM session_state WHERE key = ?1",
        params![recovery_key(request.binder_item_id)],
    )?;
    Ok(())
}

fn ensure_document_for_binder_item(conn: &mut Connection, binder_item_id: Uuid) -> AppResult<()> {
    let item_type: Option<String> = conn
        .query_row(
            "SELECT type FROM binder_items WHERE id = ?1 AND trashed_at IS NULL",
            params![binder_item_id.to_string()],
            |row| row.get(0),
        )
        .optional()?;

    match item_type.as_deref() {
        Some("folder") | Some("trash") => {
            return Err(AppError::Validation(
                "binder item does not contain a document".to_string(),
            ));
        }
        Some(_) => {}
        None => return Err(AppError::Validation("binder item not found".to_string())),
    }

    let exists = conn
        .query_row(
            "SELECT id FROM documents WHERE binder_item_id = ?1",
            params![binder_item_id.to_string()],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .is_some();

    if exists {
        return Ok(());
    }

    let now = now_iso();
    let document_id = Uuid::new_v4();
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO documents (
            id, binder_item_id, content_json, content_plain_text,
            word_count, character_count, revision, created_at, updated_at
        ) VALUES (?1, ?2, ?3, '', 0, 0, 0, ?4, ?5)",
        params![
            document_id.to_string(),
            binder_item_id.to_string(),
            EMPTY_TIPTAP_DOCUMENT,
            now,
            now,
        ],
    )?;
    tx.execute(
        "INSERT INTO document_metadata (
            document_id, keywords_json, custom_fields_json, include_in_export
        ) VALUES (?1, '[]', '{}', 1)",
        params![document_id.to_string()],
    )?;
    tx.commit()?;
    Ok(())
}

fn get_by_binder_item_id(conn: &Connection, binder_item_id: Uuid) -> AppResult<DocumentRecord> {
    conn.query_row(
        "SELECT * FROM documents WHERE binder_item_id = ?1",
        params![binder_item_id.to_string()],
        row_to_document,
    )
    .optional()?
    .ok_or_else(|| AppError::Validation("document not found".to_string()))
}

fn row_to_document(row: &Row<'_>) -> rusqlite::Result<DocumentRecord> {
    let content_json: String = row.get("content_json")?;
    let parsed_json: Value = serde_json::from_str(&content_json).unwrap_or_else(|_| json!({}));
    Ok(DocumentRecord {
        id: parse_uuid(row.get("id")?)?,
        binder_item_id: parse_uuid(row.get("binder_item_id")?)?,
        content_json: parsed_json,
        content_plain_text: row.get("content_plain_text")?,
        word_count: row.get("word_count")?,
        character_count: row.get("character_count")?,
        revision: row.get("revision")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn parse_uuid(value: String) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(&value).map_err(|error| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            error.to_string(),
        )))
    })
}

fn count_words(text: &str) -> i64 {
    text.split_whitespace()
        .filter(|word| !word.is_empty())
        .count() as i64
}

fn recovery_key(binder_item_id: Uuid) -> String {
    format!("documentRecovery:{binder_item_id}")
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::tempdir;

    use super::{get, get_recovery, record_recovery, save};
    use crate::binder;
    use crate::domain::{
        BinderItemType, CreateBinderItemRequest, CreateProjectRequest, GetDocumentRecoveryRequest,
        GetDocumentRequest, ProjectType, RecordDocumentRecoveryRequest, SaveDocumentRequest,
    };
    use crate::project;

    fn project_with_document() -> (tempfile::TempDir, uuid::Uuid) {
        let dir = tempdir().expect("tempdir");
        project::create_project(&CreateProjectRequest {
            folder_path: dir.path().display().to_string(),
            title: "Draft".to_string(),
            description: String::new(),
            project_type: ProjectType::Blank,
        })
        .expect("create project");
        let item = binder::create(
            dir.path(),
            &CreateBinderItemRequest {
                session_id: uuid::Uuid::new_v4(),
                parent_id: None,
                item_type: BinderItemType::Document,
                title: "Scene".to_string(),
            },
        )
        .expect("create binder item");
        (dir, item.id)
    }

    #[test]
    fn gets_and_saves_document() {
        let (dir, binder_item_id) = project_with_document();
        let document = get(
            dir.path(),
            &GetDocumentRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id,
            },
        )
        .expect("get document");

        let saved = save(
            dir.path(),
            &SaveDocumentRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id,
                content_json: json!({"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hello world"}]}]}),
                content_plain_text: "Hello world".to_string(),
                expected_revision: Some(document.revision),
            },
        )
        .expect("save document");

        assert_eq!(saved.word_count, 2);
        assert_eq!(saved.character_count, 11);
        assert_eq!(saved.revision, document.revision + 1);
    }

    #[test]
    fn rejects_stale_revision() {
        let (dir, binder_item_id) = project_with_document();
        let result = save(
            dir.path(),
            &SaveDocumentRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id,
                content_json: json!({"type":"doc"}),
                content_plain_text: "Text".to_string(),
                expected_revision: Some(99),
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn records_recovery_state() {
        let (dir, binder_item_id) = project_with_document();
        record_recovery(
            dir.path(),
            &RecordDocumentRecoveryRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id,
                content_json: json!({"type":"doc"}),
                content_plain_text: "Unsaved".to_string(),
                revision: 0,
            },
        )
        .expect("record recovery");

        let recovery = get_recovery(
            dir.path(),
            &GetDocumentRecoveryRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id,
            },
        )
        .expect("get recovery");

        assert_eq!(recovery.expect("recovery").content_plain_text, "Unsaved");
    }
}
