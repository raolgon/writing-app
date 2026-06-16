use std::collections::HashSet;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::database::open_project_database;
use crate::domain::{
    BinderItem, BinderItemType, CreateBinderItemRequest, DuplicateBinderItemRequest,
    MoveBinderItemRequest, RenameBinderItemRequest, ReorderBinderItemsRequest,
    RestoreBinderItemRequest, SetBinderItemExpandedRequest, TrashBinderItemRequest,
};
use crate::errors::{AppError, AppResult};
use crate::filesystem::project_db_path;
use crate::time::now_iso;

const EMPTY_TIPTAP_DOCUMENT: &str =
    r#"{"type":"doc","content":[{"type":"paragraph","content":[]}]}"#;

pub fn list(project_path: &Path, include_trashed: bool) -> AppResult<Vec<BinderItem>> {
    let conn = open_project_database(&project_db_path(project_path))?;
    let sql = if include_trashed {
        "SELECT * FROM binder_items ORDER BY parent_id IS NOT NULL, parent_id, position, title"
    } else {
        "SELECT * FROM binder_items WHERE trashed_at IS NULL ORDER BY parent_id IS NOT NULL, parent_id, position, title"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], row_to_binder_item)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn create(project_path: &Path, request: &CreateBinderItemRequest) -> AppResult<BinderItem> {
    let mut conn = open_project_database(&project_db_path(project_path))?;
    validate_title(&request.title)?;
    validate_parent(&conn, request.parent_id)?;

    let now = now_iso();
    let item_id = Uuid::new_v4();
    let document_id = Uuid::new_v4();
    let project_id = project_id(&conn)?;
    let position = next_position(&conn, request.parent_id)?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO binder_items (
            id, project_id, parent_id, type, title, synopsis, position,
            created_at, updated_at, is_expanded, is_archived
        ) VALUES (?1, ?2, ?3, ?4, ?5, '', ?6, ?7, ?8, ?9, 0)",
        params![
            item_id.to_string(),
            project_id.to_string(),
            request.parent_id.map(|id| id.to_string()),
            request.item_type.as_str(),
            request.title.trim(),
            position,
            now,
            now,
            matches!(
                request.item_type,
                BinderItemType::Folder | BinderItemType::Trash
            ) as i64,
        ],
    )?;

    if request.item_type.has_document() {
        tx.execute(
            "INSERT INTO documents (
                id, binder_item_id, content_json, content_plain_text,
                word_count, character_count, revision, created_at, updated_at
            ) VALUES (?1, ?2, ?3, '', 0, 0, 0, ?4, ?5)",
            params![
                document_id.to_string(),
                item_id.to_string(),
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
    }

    tx.commit()?;
    get_by_id(
        &open_project_database(&project_db_path(project_path))?,
        item_id,
    )
}

pub fn rename(project_path: &Path, request: &RenameBinderItemRequest) -> AppResult<BinderItem> {
    validate_title(&request.title)?;
    let conn = open_project_database(&project_db_path(project_path))?;
    let now = now_iso();
    let updated = conn.execute(
        "UPDATE binder_items SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![request.title.trim(), now, request.item_id.to_string()],
    )?;
    if updated == 0 {
        return Err(AppError::Validation("binder item not found".to_string()));
    }

    get_by_id(&conn, request.item_id)
}

pub fn set_expanded(
    project_path: &Path,
    request: &SetBinderItemExpandedRequest,
) -> AppResult<BinderItem> {
    let conn = open_project_database(&project_db_path(project_path))?;
    let updated = conn.execute(
        "UPDATE binder_items SET is_expanded = ?1, updated_at = ?2 WHERE id = ?3",
        params![
            request.is_expanded as i64,
            now_iso(),
            request.item_id.to_string()
        ],
    )?;
    if updated == 0 {
        return Err(AppError::Validation("binder item not found".to_string()));
    }

    get_by_id(&conn, request.item_id)
}

pub fn duplicate(
    project_path: &Path,
    request: &DuplicateBinderItemRequest,
) -> AppResult<BinderItem> {
    let mut conn = open_project_database(&project_db_path(project_path))?;
    let source = get_by_id(&conn, request.item_id)?;
    let tx = conn.transaction()?;
    let duplicated_id = duplicate_item_recursive(&tx, &source, source.parent_id)?;
    tx.commit()?;

    get_by_id(
        &open_project_database(&project_db_path(project_path))?,
        duplicated_id,
    )
}

pub fn move_item(
    project_path: &Path,
    request: &MoveBinderItemRequest,
) -> AppResult<Vec<BinderItem>> {
    if request.position < 0 {
        return Err(AppError::Validation(
            "position cannot be negative".to_string(),
        ));
    }

    let mut conn = open_project_database(&project_db_path(project_path))?;
    validate_parent(&conn, request.parent_id)?;
    prevent_cycle(&conn, request.item_id, request.parent_id)?;
    let tx = conn.transaction()?;
    let source_parent = parent_id(&tx, request.item_id)?;
    shift_positions_after_remove(&tx, source_parent, request.item_id)?;
    shift_positions_for_insert(&tx, request.parent_id, request.position)?;
    tx.execute(
        "UPDATE binder_items SET parent_id = ?1, position = ?2, updated_at = ?3 WHERE id = ?4",
        params![
            request.parent_id.map(|id| id.to_string()),
            request.position,
            now_iso(),
            request.item_id.to_string(),
        ],
    )?;
    tx.commit()?;
    list(project_path, false)
}

pub fn reorder(
    project_path: &Path,
    request: &ReorderBinderItemsRequest,
) -> AppResult<Vec<BinderItem>> {
    let mut conn = open_project_database(&project_db_path(project_path))?;
    validate_parent(&conn, request.parent_id)?;
    let sibling_ids = sibling_ids(&conn, request.parent_id)?;
    let requested_ids: HashSet<Uuid> = request.ordered_ids.iter().copied().collect();
    let existing_ids: HashSet<Uuid> = sibling_ids.into_iter().collect();

    if requested_ids != existing_ids {
        return Err(AppError::Validation(
            "ordered ids must exactly match current siblings".to_string(),
        ));
    }

    let tx = conn.transaction()?;
    for (position, item_id) in request.ordered_ids.iter().enumerate() {
        tx.execute(
            "UPDATE binder_items SET position = ?1, updated_at = ?2 WHERE id = ?3",
            params![position as i64, now_iso(), item_id.to_string()],
        )?;
    }
    tx.commit()?;
    list(project_path, false)
}

pub fn trash(project_path: &Path, request: &TrashBinderItemRequest) -> AppResult<Vec<BinderItem>> {
    let conn = open_project_database(&project_db_path(project_path))?;
    let now = now_iso();
    let updated = conn.execute(
        "UPDATE binder_items SET trashed_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![now, now, request.item_id.to_string()],
    )?;
    if updated == 0 {
        return Err(AppError::Validation("binder item not found".to_string()));
    }

    list(project_path, false)
}

pub fn restore(
    project_path: &Path,
    request: &RestoreBinderItemRequest,
) -> AppResult<Vec<BinderItem>> {
    let conn = open_project_database(&project_db_path(project_path))?;
    let updated = conn.execute(
        "UPDATE binder_items SET trashed_at = NULL, updated_at = ?1 WHERE id = ?2",
        params![now_iso(), request.item_id.to_string()],
    )?;
    if updated == 0 {
        return Err(AppError::Validation("binder item not found".to_string()));
    }

    list(project_path, false)
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
        item_type: BinderItemType::try_from(item_type.as_str()).map_err(|error| {
            to_sql_error(std::io::Error::new(std::io::ErrorKind::InvalidData, error))
        })?,
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

fn project_id(conn: &Connection) -> AppResult<Uuid> {
    let id: String = conn.query_row("SELECT id FROM projects LIMIT 1", [], |row| row.get(0))?;
    Uuid::parse_str(&id).map_err(|error| AppError::ProjectCorrupt(error.to_string()))
}

fn get_by_id(conn: &Connection, item_id: Uuid) -> AppResult<BinderItem> {
    conn.query_row(
        "SELECT * FROM binder_items WHERE id = ?1",
        params![item_id.to_string()],
        row_to_binder_item,
    )
    .optional()?
    .ok_or_else(|| AppError::Validation("binder item not found".to_string()))
}

fn parent_id(conn: &Connection, item_id: Uuid) -> AppResult<Option<Uuid>> {
    let parent: Option<String> = conn.query_row(
        "SELECT parent_id FROM binder_items WHERE id = ?1",
        params![item_id.to_string()],
        |row| row.get(0),
    )?;
    parent
        .map(|id| Uuid::parse_str(&id).map_err(|error| AppError::ProjectCorrupt(error.to_string())))
        .transpose()
}

fn validate_title(title: &str) -> AppResult<()> {
    if title.trim().is_empty() {
        return Err(AppError::Validation(
            "binder item title cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn validate_parent(conn: &Connection, parent_id: Option<Uuid>) -> AppResult<()> {
    if let Some(parent_id) = parent_id {
        let parent_type: Option<String> = conn
            .query_row(
                "SELECT type FROM binder_items WHERE id = ?1 AND trashed_at IS NULL",
                params![parent_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        if parent_type.is_none() {
            return Err(AppError::Validation(
                "parent binder item not found".to_string(),
            ));
        }
    }
    Ok(())
}

fn next_position(conn: &Connection, parent_id: Option<Uuid>) -> AppResult<i64> {
    let position: Option<i64> = match parent_id {
        Some(parent_id) => conn.query_row(
            "SELECT MAX(position) + 1 FROM binder_items WHERE parent_id = ?1 AND trashed_at IS NULL",
            params![parent_id.to_string()],
            |row| row.get(0),
        )?,
        None => conn.query_row(
            "SELECT MAX(position) + 1 FROM binder_items WHERE parent_id IS NULL AND trashed_at IS NULL",
            [],
            |row| row.get(0),
        )?,
    };

    Ok(position.unwrap_or(0))
}

fn prevent_cycle(
    conn: &Connection,
    item_id: Uuid,
    target_parent_id: Option<Uuid>,
) -> AppResult<()> {
    let mut current = target_parent_id;
    while let Some(current_parent_id) = current {
        if current_parent_id == item_id {
            return Err(AppError::Validation(
                "cannot move a binder item inside itself".to_string(),
            ));
        }
        current = parent_id(conn, current_parent_id)?;
    }
    Ok(())
}

fn sibling_ids(conn: &Connection, parent_id: Option<Uuid>) -> AppResult<Vec<Uuid>> {
    match parent_id {
        Some(parent_id) => {
            let mut stmt = conn.prepare(
                "SELECT id FROM binder_items WHERE parent_id = ?1 AND trashed_at IS NULL ORDER BY position",
            )?;
            let rows = stmt.query_map(params![parent_id.to_string()], |row| {
                parse_uuid(row.get(0)?)
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT id FROM binder_items WHERE parent_id IS NULL AND trashed_at IS NULL ORDER BY position",
            )?;
            let rows = stmt.query_map([], |row| parse_uuid(row.get(0)?))?;
            rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
        }
    }
}

fn shift_positions_after_remove(
    conn: &Connection,
    parent_id: Option<Uuid>,
    item_id: Uuid,
) -> AppResult<()> {
    let old_position: i64 = conn.query_row(
        "SELECT position FROM binder_items WHERE id = ?1",
        params![item_id.to_string()],
        |row| row.get(0),
    )?;
    match parent_id {
        Some(parent_id) => {
            conn.execute(
                "UPDATE binder_items SET position = position - 1 WHERE parent_id = ?1 AND position > ?2 AND trashed_at IS NULL",
                params![parent_id.to_string(), old_position],
            )?;
        }
        None => {
            conn.execute(
                "UPDATE binder_items SET position = position - 1 WHERE parent_id IS NULL AND position > ?1 AND trashed_at IS NULL",
                params![old_position],
            )?;
        }
    }
    Ok(())
}

fn shift_positions_for_insert(
    conn: &Connection,
    parent_id: Option<Uuid>,
    position: i64,
) -> AppResult<()> {
    match parent_id {
        Some(parent_id) => {
            conn.execute(
                "UPDATE binder_items SET position = position + 1 WHERE parent_id = ?1 AND position >= ?2 AND trashed_at IS NULL",
                params![parent_id.to_string(), position],
            )?;
        }
        None => {
            conn.execute(
                "UPDATE binder_items SET position = position + 1 WHERE parent_id IS NULL AND position >= ?1 AND trashed_at IS NULL",
                params![position],
            )?;
        }
    }
    Ok(())
}

fn duplicate_item_recursive(
    conn: &Connection,
    source: &BinderItem,
    parent_id: Option<Uuid>,
) -> AppResult<Uuid> {
    let new_id = Uuid::new_v4();
    let now = now_iso();
    let position = next_position(conn, parent_id)?;
    conn.execute(
        "INSERT INTO binder_items (
            id, project_id, parent_id, type, title, synopsis, position, icon, color_label,
            status, created_at, updated_at, is_expanded, is_archived
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            new_id.to_string(),
            source.project_id.to_string(),
            parent_id.map(|id| id.to_string()),
            source.item_type.as_str(),
            format!("{} Copy", source.title),
            source.synopsis,
            position,
            source.icon,
            source.color_label,
            source.status,
            now,
            now,
            source.is_expanded as i64,
            source.is_archived as i64,
        ],
    )?;

    if source.item_type.has_document() {
        duplicate_document(conn, source.id, new_id, &now)?;
    }

    let children = children(conn, source.id)?;
    for child in children {
        duplicate_item_recursive(conn, &child, Some(new_id))?;
    }

    Ok(new_id)
}

fn duplicate_document(
    conn: &Connection,
    source_item_id: Uuid,
    new_item_id: Uuid,
    now: &str,
) -> AppResult<()> {
    let source_document: Option<(String, String, i64, i64)> = conn
        .query_row(
            "SELECT content_json, content_plain_text, word_count, character_count
             FROM documents WHERE binder_item_id = ?1",
            params![source_item_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()?;
    let (content_json, content_plain_text, word_count, character_count) =
        source_document.unwrap_or_else(|| (EMPTY_TIPTAP_DOCUMENT.to_string(), String::new(), 0, 0));
    let document_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO documents (
            id, binder_item_id, content_json, content_plain_text, word_count,
            character_count, revision, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8)",
        params![
            document_id.to_string(),
            new_item_id.to_string(),
            content_json,
            content_plain_text,
            word_count,
            character_count,
            now,
            now,
        ],
    )?;
    conn.execute(
        "INSERT INTO document_metadata (
            document_id, keywords_json, custom_fields_json, include_in_export
        ) VALUES (?1, '[]', '{}', 1)",
        params![document_id.to_string()],
    )?;
    Ok(())
}

fn children(conn: &Connection, parent_id: Uuid) -> AppResult<Vec<BinderItem>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM binder_items WHERE parent_id = ?1 AND trashed_at IS NULL ORDER BY position",
    )?;
    let rows = stmt.query_map(params![parent_id.to_string()], row_to_binder_item)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{create, list, move_item, reorder, restore, trash};
    use crate::domain::{
        CreateBinderItemRequest, CreateProjectRequest, MoveBinderItemRequest, ProjectType,
        ReorderBinderItemsRequest, RestoreBinderItemRequest, TrashBinderItemRequest,
    };
    use crate::project;

    fn project_dir() -> tempfile::TempDir {
        let dir = tempdir().expect("tempdir");
        project::create_project(&CreateProjectRequest {
            folder_path: dir.path().display().to_string(),
            title: "Draft".to_string(),
            description: String::new(),
            project_type: ProjectType::Blank,
        })
        .expect("create project");
        dir
    }

    #[test]
    fn creates_binder_items() {
        let dir = project_dir();
        create(
            dir.path(),
            &CreateBinderItemRequest {
                session_id: uuid::Uuid::new_v4(),
                parent_id: None,
                item_type: crate::domain::BinderItemType::Document,
                title: "Scene".to_string(),
            },
        )
        .expect("create item");

        assert_eq!(list(dir.path(), false).expect("list").len(), 1);
    }

    #[test]
    fn reorders_siblings() {
        let dir = project_dir();
        let first = create(
            dir.path(),
            &CreateBinderItemRequest {
                session_id: uuid::Uuid::new_v4(),
                parent_id: None,
                item_type: crate::domain::BinderItemType::Document,
                title: "A".to_string(),
            },
        )
        .expect("first");
        let second = create(
            dir.path(),
            &CreateBinderItemRequest {
                session_id: uuid::Uuid::new_v4(),
                parent_id: None,
                item_type: crate::domain::BinderItemType::Document,
                title: "B".to_string(),
            },
        )
        .expect("second");

        reorder(
            dir.path(),
            &ReorderBinderItemsRequest {
                session_id: uuid::Uuid::new_v4(),
                parent_id: None,
                ordered_ids: vec![second.id, first.id],
            },
        )
        .expect("reorder");

        let items = list(dir.path(), false).expect("list");
        assert_eq!(items[0].title, "B");
        assert_eq!(items[1].title, "A");
    }

    #[test]
    fn moves_item_under_parent() {
        let dir = project_dir();
        let folder = create(
            dir.path(),
            &CreateBinderItemRequest {
                session_id: uuid::Uuid::new_v4(),
                parent_id: None,
                item_type: crate::domain::BinderItemType::Folder,
                title: "Folder".to_string(),
            },
        )
        .expect("folder");
        let doc = create(
            dir.path(),
            &CreateBinderItemRequest {
                session_id: uuid::Uuid::new_v4(),
                parent_id: None,
                item_type: crate::domain::BinderItemType::Document,
                title: "Scene".to_string(),
            },
        )
        .expect("doc");

        move_item(
            dir.path(),
            &MoveBinderItemRequest {
                session_id: uuid::Uuid::new_v4(),
                item_id: doc.id,
                parent_id: Some(folder.id),
                position: 0,
            },
        )
        .expect("move");

        let items = list(dir.path(), false).expect("list");
        let moved = items.iter().find(|item| item.id == doc.id).expect("moved");
        assert_eq!(moved.parent_id, Some(folder.id));
    }

    #[test]
    fn trashes_and_restores_item() {
        let dir = project_dir();
        let doc = create(
            dir.path(),
            &CreateBinderItemRequest {
                session_id: uuid::Uuid::new_v4(),
                parent_id: None,
                item_type: crate::domain::BinderItemType::Document,
                title: "Scene".to_string(),
            },
        )
        .expect("doc");

        trash(
            dir.path(),
            &TrashBinderItemRequest {
                session_id: uuid::Uuid::new_v4(),
                item_id: doc.id,
            },
        )
        .expect("trash");
        assert!(list(dir.path(), false).expect("list").is_empty());

        restore(
            dir.path(),
            &RestoreBinderItemRequest {
                session_id: uuid::Uuid::new_v4(),
                item_id: doc.id,
            },
        )
        .expect("restore");
        assert_eq!(list(dir.path(), false).expect("list").len(), 1);
    }
}
