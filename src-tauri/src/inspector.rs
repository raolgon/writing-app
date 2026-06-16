use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::json;
use uuid::Uuid;

use crate::database::open_project_database;
use crate::domain::{
    BinderItem, CreateSnapshotRequest, DocumentMetadata, DocumentRecord, GetInspectorDataRequest,
    InspectorData, ProjectNote, RestoreSnapshotRequest, SaveBinderSynopsisRequest,
    SaveDocumentMetadataRequest, SaveProjectNoteRequest, Snapshot,
};
use crate::errors::{AppError, AppResult};
use crate::filesystem::project_db_path;
use crate::time::now_iso;

pub fn get_data(
    project_path: &Path,
    request: &GetInspectorDataRequest,
) -> AppResult<InspectorData> {
    let conn = open_project_database(&project_db_path(project_path))?;
    let document_id = document_id_for_binder_item(&conn, request.binder_item_id)?;

    Ok(InspectorData {
        metadata: metadata_by_document_id(&conn, document_id)?,
        notes: notes_for_binder_item(&conn, request.binder_item_id)?,
        snapshots: snapshots_for_document(&conn, document_id)?,
    })
}

pub fn save_synopsis(
    project_path: &Path,
    request: &SaveBinderSynopsisRequest,
) -> AppResult<BinderItem> {
    let conn = open_project_database(&project_db_path(project_path))?;
    let now = now_iso();
    let updated = conn.execute(
        "UPDATE binder_items SET synopsis = ?1, updated_at = ?2 WHERE id = ?3",
        params![request.synopsis, now, request.item_id.to_string()],
    )?;
    if updated == 0 {
        return Err(AppError::Validation("binder item not found".to_string()));
    }

    binder_item_by_id(&conn, request.item_id)
}

pub fn save_metadata(
    project_path: &Path,
    request: &SaveDocumentMetadataRequest,
) -> AppResult<DocumentMetadata> {
    let conn = open_project_database(&project_db_path(project_path))?;
    let document_id = document_id_for_binder_item(&conn, request.binder_item_id)?;
    let keywords = normalize_keywords(&request.keywords);
    let custom_fields = if request.custom_fields.is_null() {
        json!({})
    } else {
        request.custom_fields.clone()
    };

    conn.execute(
        "INSERT INTO document_metadata (
            document_id, label, status, target_word_count, keywords_json,
            custom_fields_json, include_in_export
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(document_id) DO UPDATE SET
            label = excluded.label,
            status = excluded.status,
            target_word_count = excluded.target_word_count,
            keywords_json = excluded.keywords_json,
            custom_fields_json = excluded.custom_fields_json,
            include_in_export = excluded.include_in_export",
        params![
            document_id.to_string(),
            clean_optional_text(request.label.as_deref()),
            clean_optional_text(request.status.as_deref()),
            request.target_word_count,
            serde_json::to_string(&keywords)?,
            serde_json::to_string(&custom_fields)?,
            request.include_in_export as i64,
        ],
    )?;

    metadata_by_document_id(&conn, document_id)?
        .ok_or_else(|| AppError::Validation("document metadata not found".to_string()))
}

pub fn save_note(project_path: &Path, request: &SaveProjectNoteRequest) -> AppResult<ProjectNote> {
    validate_note_title(&request.title)?;
    let conn = open_project_database(&project_db_path(project_path))?;
    if let Some(binder_item_id) = request.binder_item_id {
        validate_binder_item(&conn, binder_item_id)?;
    }

    let project_id = project_id(&conn)?;
    let now = now_iso();

    match request.id {
        Some(note_id) => {
            let updated = conn.execute(
                "UPDATE project_notes
                 SET binder_item_id = ?1, title = ?2, content = ?3, updated_at = ?4
                 WHERE id = ?5",
                params![
                    request.binder_item_id.map(|id| id.to_string()),
                    request.title.trim(),
                    request.content,
                    now,
                    note_id.to_string(),
                ],
            )?;
            if updated == 0 {
                return Err(AppError::Validation("note not found".to_string()));
            }
            note_by_id(&conn, note_id)
        }
        None => {
            let note_id = Uuid::new_v4();
            conn.execute(
                "INSERT INTO project_notes (
                    id, project_id, binder_item_id, title, content, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    note_id.to_string(),
                    project_id.to_string(),
                    request.binder_item_id.map(|id| id.to_string()),
                    request.title.trim(),
                    request.content,
                    now,
                    now,
                ],
            )?;
            note_by_id(&conn, note_id)
        }
    }
}

pub fn create_snapshot(
    project_path: &Path,
    request: &CreateSnapshotRequest,
) -> AppResult<Snapshot> {
    validate_snapshot_name(&request.name)?;
    let conn = open_project_database(&project_db_path(project_path))?;
    let document = document_by_binder_item_id(&conn, request.binder_item_id)?;
    let snapshot_id = Uuid::new_v4();
    let now = now_iso();

    conn.execute(
        "INSERT INTO snapshots (
            id, document_id, name, content_json, content_plain_text, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            snapshot_id.to_string(),
            document.id.to_string(),
            request.name.trim(),
            serde_json::to_string(&document.content_json)?,
            document.content_plain_text,
            now,
        ],
    )?;

    snapshot_by_id(&conn, snapshot_id)
}

pub fn restore_snapshot(
    project_path: &Path,
    request: &RestoreSnapshotRequest,
) -> AppResult<DocumentRecord> {
    let mut conn = open_project_database(&project_db_path(project_path))?;
    let snapshot = snapshot_by_id(&conn, request.snapshot_id)?;
    let now = now_iso();
    let word_count = count_words(&snapshot.content_plain_text);
    let character_count = snapshot.content_plain_text.chars().count() as i64;
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE documents
         SET content_json = ?1,
             content_plain_text = ?2,
             word_count = ?3,
             character_count = ?4,
             revision = revision + 1,
             updated_at = ?5
         WHERE id = ?6",
        params![
            serde_json::to_string(&snapshot.content_json)?,
            snapshot.content_plain_text,
            word_count,
            character_count,
            now,
            snapshot.document_id.to_string(),
        ],
    )?;
    tx.execute(
        "UPDATE binder_items
         SET updated_at = ?1
         WHERE id = (SELECT binder_item_id FROM documents WHERE id = ?2)",
        params![now, snapshot.document_id.to_string()],
    )?;
    tx.commit()?;

    document_by_id(&conn, snapshot.document_id)
}

fn document_id_for_binder_item(conn: &Connection, binder_item_id: Uuid) -> AppResult<Uuid> {
    document_by_binder_item_id(conn, binder_item_id).map(|document| document.id)
}

fn document_by_binder_item_id(
    conn: &Connection,
    binder_item_id: Uuid,
) -> AppResult<DocumentRecord> {
    conn.query_row(
        "SELECT * FROM documents WHERE binder_item_id = ?1",
        params![binder_item_id.to_string()],
        row_to_document,
    )
    .optional()?
    .ok_or_else(|| AppError::Validation("document not found".to_string()))
}

fn document_by_id(conn: &Connection, document_id: Uuid) -> AppResult<DocumentRecord> {
    conn.query_row(
        "SELECT * FROM documents WHERE id = ?1",
        params![document_id.to_string()],
        row_to_document,
    )
    .optional()?
    .ok_or_else(|| AppError::Validation("document not found".to_string()))
}

fn metadata_by_document_id(
    conn: &Connection,
    document_id: Uuid,
) -> AppResult<Option<DocumentMetadata>> {
    conn.query_row(
        "SELECT * FROM document_metadata WHERE document_id = ?1",
        params![document_id.to_string()],
        row_to_metadata,
    )
    .optional()
    .map_err(AppError::from)
}

fn notes_for_binder_item(conn: &Connection, binder_item_id: Uuid) -> AppResult<Vec<ProjectNote>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM project_notes
         WHERE binder_item_id = ?1
         ORDER BY updated_at DESC, created_at DESC",
    )?;
    let rows = stmt.query_map(params![binder_item_id.to_string()], row_to_note)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

fn snapshots_for_document(conn: &Connection, document_id: Uuid) -> AppResult<Vec<Snapshot>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM snapshots
         WHERE document_id = ?1
         ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![document_id.to_string()], row_to_snapshot)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

fn note_by_id(conn: &Connection, note_id: Uuid) -> AppResult<ProjectNote> {
    conn.query_row(
        "SELECT * FROM project_notes WHERE id = ?1",
        params![note_id.to_string()],
        row_to_note,
    )
    .optional()?
    .ok_or_else(|| AppError::Validation("note not found".to_string()))
}

fn snapshot_by_id(conn: &Connection, snapshot_id: Uuid) -> AppResult<Snapshot> {
    conn.query_row(
        "SELECT * FROM snapshots WHERE id = ?1",
        params![snapshot_id.to_string()],
        row_to_snapshot,
    )
    .optional()?
    .ok_or_else(|| AppError::Validation("snapshot not found".to_string()))
}

fn binder_item_by_id(conn: &Connection, item_id: Uuid) -> AppResult<BinderItem> {
    conn.query_row(
        "SELECT * FROM binder_items WHERE id = ?1",
        params![item_id.to_string()],
        row_to_binder_item,
    )
    .optional()?
    .ok_or_else(|| AppError::Validation("binder item not found".to_string()))
}

fn validate_binder_item(conn: &Connection, binder_item_id: Uuid) -> AppResult<()> {
    let exists = conn
        .query_row(
            "SELECT id FROM binder_items WHERE id = ?1 AND trashed_at IS NULL",
            params![binder_item_id.to_string()],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .is_some();

    if exists {
        Ok(())
    } else {
        Err(AppError::Validation("binder item not found".to_string()))
    }
}

fn project_id(conn: &Connection) -> AppResult<Uuid> {
    let id: String = conn.query_row("SELECT id FROM projects LIMIT 1", [], |row| row.get(0))?;
    parse_uuid(id).map_err(AppError::from)
}

fn clean_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn normalize_keywords(keywords: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for keyword in keywords {
        let trimmed = keyword.trim();
        if !trimmed.is_empty() && !normalized.iter().any(|item| item == trimmed) {
            normalized.push(trimmed.to_string());
        }
    }
    normalized
}

fn validate_note_title(title: &str) -> AppResult<()> {
    if title.trim().is_empty() {
        return Err(AppError::Validation(
            "note title cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn validate_snapshot_name(name: &str) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::Validation(
            "snapshot name cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn count_words(text: &str) -> i64 {
    text.split_whitespace()
        .filter(|word| !word.is_empty())
        .count() as i64
}

fn row_to_metadata(row: &Row<'_>) -> rusqlite::Result<DocumentMetadata> {
    let keywords_json: String = row.get("keywords_json")?;
    let custom_fields_json: String = row.get("custom_fields_json")?;
    Ok(DocumentMetadata {
        document_id: parse_uuid(row.get("document_id")?)?,
        label: row.get("label")?,
        status: row.get("status")?,
        target_word_count: row.get("target_word_count")?,
        keywords: serde_json::from_str(&keywords_json).unwrap_or_default(),
        custom_fields: serde_json::from_str(&custom_fields_json).unwrap_or_else(|_| json!({})),
        include_in_export: row.get::<_, i64>("include_in_export")? != 0,
    })
}

fn row_to_note(row: &Row<'_>) -> rusqlite::Result<ProjectNote> {
    Ok(ProjectNote {
        id: parse_uuid(row.get("id")?)?,
        project_id: parse_uuid(row.get("project_id")?)?,
        binder_item_id: row
            .get::<_, Option<String>>("binder_item_id")?
            .map(parse_uuid)
            .transpose()?,
        title: row.get("title")?,
        content: row.get("content")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn row_to_snapshot(row: &Row<'_>) -> rusqlite::Result<Snapshot> {
    let content_json: String = row.get("content_json")?;
    Ok(Snapshot {
        id: parse_uuid(row.get("id")?)?,
        document_id: parse_uuid(row.get("document_id")?)?,
        name: row.get("name")?,
        content_json: serde_json::from_str(&content_json).unwrap_or_else(|_| json!({})),
        content_plain_text: row.get("content_plain_text")?,
        created_at: row.get("created_at")?,
    })
}

fn row_to_document(row: &Row<'_>) -> rusqlite::Result<DocumentRecord> {
    let content_json: String = row.get("content_json")?;
    Ok(DocumentRecord {
        id: parse_uuid(row.get("id")?)?,
        binder_item_id: parse_uuid(row.get("binder_item_id")?)?,
        content_json: serde_json::from_str(&content_json).unwrap_or_else(|_| json!({})),
        content_plain_text: row.get("content_plain_text")?,
        word_count: row.get("word_count")?,
        character_count: row.get("character_count")?,
        revision: row.get("revision")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn row_to_binder_item(row: &Row<'_>) -> rusqlite::Result<BinderItem> {
    let item_type: String = row.get("type")?;
    Ok(BinderItem {
        id: parse_uuid(row.get("id")?)?,
        project_id: parse_uuid(row.get("project_id")?)?,
        parent_id: row
            .get::<_, Option<String>>("parent_id")?
            .map(parse_uuid)
            .transpose()?,
        item_type: crate::domain::BinderItemType::try_from(item_type.as_str()).map_err(
            |error| to_sql_error(std::io::Error::new(std::io::ErrorKind::InvalidData, error)),
        )?,
        title: row.get("title")?,
        synopsis: row.get("synopsis")?,
        position: row.get("position")?,
        icon: row.get("icon")?,
        color_label: row.get("color_label")?,
        status: row.get("status")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        is_expanded: row.get::<_, i64>("is_expanded")? != 0,
        is_archived: row.get::<_, i64>("is_archived")? != 0,
        trashed_at: row.get("trashed_at")?,
    })
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
    use serde_json::json;
    use tempfile::tempdir;

    use super::{
        create_snapshot, get_data, restore_snapshot, save_metadata, save_note, save_synopsis,
    };
    use crate::binder;
    use crate::documents;
    use crate::domain::{
        BinderItemType, CreateBinderItemRequest, CreateProjectRequest, CreateSnapshotRequest,
        GetDocumentRequest, GetInspectorDataRequest, ProjectType, RestoreSnapshotRequest,
        SaveBinderSynopsisRequest, SaveDocumentMetadataRequest, SaveDocumentRequest,
        SaveProjectNoteRequest,
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
    fn saves_inspector_metadata_and_note() {
        let (dir, binder_item_id) = project_with_document();

        let item = save_synopsis(
            dir.path(),
            &SaveBinderSynopsisRequest {
                session_id: uuid::Uuid::new_v4(),
                item_id: binder_item_id,
                synopsis: "Short summary".to_string(),
            },
        )
        .expect("save synopsis");
        assert_eq!(item.synopsis, "Short summary");

        let metadata = save_metadata(
            dir.path(),
            &SaveDocumentMetadataRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id,
                label: Some("A".to_string()),
                status: Some("Draft".to_string()),
                target_word_count: Some(1200),
                keywords: vec!["one".to_string(), "one".to_string(), "two".to_string()],
                custom_fields: json!({}),
                include_in_export: true,
            },
        )
        .expect("save metadata");
        assert_eq!(metadata.keywords, vec!["one", "two"]);

        let note = save_note(
            dir.path(),
            &SaveProjectNoteRequest {
                session_id: uuid::Uuid::new_v4(),
                id: None,
                binder_item_id: Some(binder_item_id),
                title: "Notas".to_string(),
                content: "Private note".to_string(),
            },
        )
        .expect("save note");
        assert_eq!(note.content, "Private note");

        let data = get_data(
            dir.path(),
            &GetInspectorDataRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id,
            },
        )
        .expect("get inspector data");
        assert_eq!(data.notes.len(), 1);
        assert_eq!(
            data.metadata.expect("metadata").status,
            Some("Draft".to_string())
        );
    }

    #[test]
    fn creates_and_restores_snapshots() {
        let (dir, binder_item_id) = project_with_document();
        let document = documents::get(
            dir.path(),
            &GetDocumentRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id,
            },
        )
        .expect("get document");
        documents::save(
            dir.path(),
            &SaveDocumentRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id,
                content_json: json!({"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Version one"}]}]}),
                content_plain_text: "Version one".to_string(),
                expected_revision: Some(document.revision),
            },
        )
        .expect("save document");
        let snapshot = create_snapshot(
            dir.path(),
            &CreateSnapshotRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id,
                name: "Before edits".to_string(),
            },
        )
        .expect("create snapshot");
        let document = documents::get(
            dir.path(),
            &GetDocumentRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id,
            },
        )
        .expect("get document");
        documents::save(
            dir.path(),
            &SaveDocumentRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id,
                content_json: json!({"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Version two"}]}]}),
                content_plain_text: "Version two".to_string(),
                expected_revision: Some(document.revision),
            },
        )
        .expect("save document");

        let restored = restore_snapshot(
            dir.path(),
            &RestoreSnapshotRequest {
                session_id: uuid::Uuid::new_v4(),
                snapshot_id: snapshot.id,
            },
        )
        .expect("restore snapshot");

        assert_eq!(restored.content_plain_text, "Version one");
    }
}
