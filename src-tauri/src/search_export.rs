use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::database::open_project_database;
use crate::domain::{
    BinderItemType, ExportFormat, ExportProjectRequest, ExportScope, ExportedFile,
    SearchProjectRequest, SearchResult,
};
use crate::errors::{AppError, AppResult};
use crate::filesystem::project_db_path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportDocument {
    binder_item_id: Uuid,
    title: String,
    path: Vec<String>,
    content_plain_text: String,
    word_count: i64,
}

#[derive(Debug, Clone)]
struct SearchRow {
    binder_item_id: Uuid,
    title: String,
    item_type: String,
    snippet: String,
    updated_at: String,
}

pub fn search(project_path: &Path, request: &SearchProjectRequest) -> AppResult<Vec<SearchResult>> {
    let conn = open_project_database(&project_db_path(project_path))?;
    rebuild_index(&conn)?;
    let query = fts_query(&request.query);
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare(
        "SELECT
            s.binder_item_id,
            b.title,
            b.type,
            b.updated_at,
            snippet(search_index, -1, '', '', '...', 16) AS snippet
         FROM search_index s
         JOIN binder_items b ON b.id = s.binder_item_id
         WHERE search_index MATCH ?1 AND b.trashed_at IS NULL
         ORDER BY rank",
    )?;
    let rows = stmt
        .query_map(params![query], row_to_search_row)?
        .collect::<Result<Vec<_>, _>>()?;
    rows.into_iter()
        .map(|row| {
            Ok(SearchResult {
                binder_item_id: row.binder_item_id,
                title: row.title,
                item_type: BinderItemType::try_from(row.item_type.as_str())
                    .map_err(AppError::Validation)?,
                path: binder_path(&conn, row.binder_item_id)?,
                snippet: row.snippet,
                updated_at: row.updated_at,
            })
        })
        .collect()
}

pub fn export_project(
    project_path: &Path,
    request: &ExportProjectRequest,
) -> AppResult<ExportedFile> {
    let conn = open_project_database(&project_db_path(project_path))?;
    let project_title: String =
        conn.query_row("SELECT title FROM projects LIMIT 1", [], |row| row.get(0))?;
    let documents = export_documents(&conn, request)?;
    let content = match request.format {
        ExportFormat::Txt => render_txt(&documents, request),
        ExportFormat::Markdown => render_markdown(&documents, request),
        ExportFormat::Html => render_html(&documents, request),
        ExportFormat::Json => serde_json::to_string_pretty(&json!({
            "projectTitle": project_title,
            "scope": request.scope,
            "documents": documents,
        }))?,
    };

    Ok(ExportedFile {
        file_name: format!(
            "{}.{}",
            sanitize_file_name(&project_title),
            request.format.extension()
        ),
        mime_type: request.format.mime_type().to_string(),
        content,
    })
}

fn rebuild_index(conn: &Connection) -> AppResult<()> {
    conn.execute("DELETE FROM search_index", [])?;
    let mut stmt = conn.prepare(
        "SELECT
            b.id,
            b.title,
            b.synopsis,
            COALESCE(d.content_plain_text, ''),
            COALESCE((
                SELECT GROUP_CONCAT(n.title || ' ' || n.content, ' ')
                FROM project_notes n
                WHERE n.binder_item_id = b.id
            ), ''),
            COALESCE(m.keywords_json, '[]')
         FROM binder_items b
         LEFT JOIN documents d ON d.binder_item_id = b.id
         LEFT JOIN document_metadata m ON m.document_id = d.id
         WHERE b.trashed_at IS NULL",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;

    for row in rows {
        let (id, title, synopsis, content, notes, keywords_json) = row?;
        let keywords = keywords_from_json(&keywords_json).join(" ");
        conn.execute(
            "INSERT INTO search_index (
                binder_item_id, title, synopsis, content, notes, keywords
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, title, synopsis, content, notes, keywords],
        )?;
    }

    Ok(())
}

fn row_to_search_row(row: &Row<'_>) -> rusqlite::Result<SearchRow> {
    Ok(SearchRow {
        binder_item_id: parse_uuid(row.get("binder_item_id")?)?,
        title: row.get("title")?,
        item_type: row.get("type")?,
        snippet: row.get("snippet")?,
        updated_at: row.get("updated_at")?,
    })
}

fn export_documents(
    conn: &Connection,
    request: &ExportProjectRequest,
) -> AppResult<Vec<ExportDocument>> {
    match request.scope {
        ExportScope::Document => {
            let id = request.binder_item_id.ok_or_else(|| {
                AppError::Validation("document export requires binder item".to_string())
            })?;
            document_by_binder_item(conn, id).map(|document| vec![document])
        }
        ExportScope::Folder => {
            let id = request.binder_item_id.ok_or_else(|| {
                AppError::Validation("folder export requires binder item".to_string())
            })?;
            let mut ids = Vec::new();
            collect_document_ids(conn, Some(id), &mut ids)?;
            ids.into_iter()
                .map(|id| document_by_binder_item(conn, id))
                .collect()
        }
        ExportScope::Included => {
            let mut stmt = conn.prepare(
                "SELECT b.id
                 FROM binder_items b
                 JOIN documents d ON d.binder_item_id = b.id
                 JOIN document_metadata m ON m.document_id = d.id
                 WHERE b.trashed_at IS NULL AND m.include_in_export = 1
                 ORDER BY b.parent_id IS NOT NULL, b.parent_id, b.position",
            )?;
            let rows = stmt.query_map([], |row| parse_uuid(row.get(0)?))?;
            rows.collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(|id| document_by_binder_item(conn, id))
                .collect()
        }
    }
}

fn collect_document_ids(
    conn: &Connection,
    parent_id: Option<Uuid>,
    ids: &mut Vec<Uuid>,
) -> AppResult<()> {
    let mut stmt = match parent_id {
        Some(_) => conn.prepare(
            "SELECT id, type FROM binder_items
             WHERE parent_id = ?1 AND trashed_at IS NULL
             ORDER BY position",
        )?,
        None => conn.prepare(
            "SELECT id, type FROM binder_items
             WHERE parent_id IS NULL AND trashed_at IS NULL
             ORDER BY position",
        )?,
    };
    let rows: Vec<(Uuid, String)> = if let Some(parent_id) = parent_id {
        stmt.query_map(params![parent_id.to_string()], |row| {
            Ok((parse_uuid(row.get(0)?)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?
    } else {
        stmt.query_map([], |row| {
            Ok((parse_uuid(row.get(0)?)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?
    };

    for (id, item_type) in rows {
        if item_type == "folder" {
            collect_document_ids(conn, Some(id), ids)?;
        } else {
            ids.push(id);
        }
    }

    Ok(())
}

fn document_by_binder_item(conn: &Connection, id: Uuid) -> AppResult<ExportDocument> {
    conn.query_row(
        "SELECT b.title, d.content_plain_text, d.word_count
         FROM binder_items b
         JOIN documents d ON d.binder_item_id = b.id
         WHERE b.id = ?1 AND b.trashed_at IS NULL",
        params![id.to_string()],
        |row| {
            Ok(ExportDocument {
                binder_item_id: id,
                title: row.get(0)?,
                path: binder_path(conn, id).map_err(to_sql_error)?,
                content_plain_text: row.get(1)?,
                word_count: row.get(2)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| AppError::Validation("export document not found".to_string()))
}

fn render_txt(documents: &[ExportDocument], request: &ExportProjectRequest) -> String {
    documents
        .iter()
        .map(|document| {
            let mut parts = Vec::new();
            if request.include_titles {
                parts.push(document.title.clone());
            }
            parts.push(document.content_plain_text.clone());
            parts.join("\n\n")
        })
        .collect::<Vec<_>>()
        .join(if request.separate_scenes {
            "\n\n---\n\n"
        } else {
            "\n\n"
        })
}

fn render_markdown(documents: &[ExportDocument], request: &ExportProjectRequest) -> String {
    documents
        .iter()
        .map(|document| {
            let mut parts = Vec::new();
            if request.include_titles {
                parts.push(format!("# {}", document.title));
            }
            parts.push(document.content_plain_text.clone());
            parts.join("\n\n")
        })
        .collect::<Vec<_>>()
        .join(if request.separate_scenes {
            "\n\n---\n\n"
        } else {
            "\n\n"
        })
}

fn render_html(documents: &[ExportDocument], request: &ExportProjectRequest) -> String {
    let body = documents
        .iter()
        .map(|document| {
            let title = if request.include_titles {
                format!("<h1>{}</h1>", escape_html(&document.title))
            } else {
                String::new()
            };
            let paragraphs = document
                .content_plain_text
                .split("\n\n")
                .filter(|paragraph| !paragraph.trim().is_empty())
                .map(|paragraph| format!("<p>{}</p>", escape_html(paragraph.trim())))
                .collect::<Vec<_>>()
                .join("\n");
            format!("<section>\n{title}\n{paragraphs}\n</section>")
        })
        .collect::<Vec<_>>()
        .join("\n<hr />\n");
    format!("<!doctype html>\n<html><body>\n{body}\n</body></html>")
}

fn fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|token| {
            token
                .chars()
                .filter(|character| character.is_alphanumeric())
                .collect::<String>()
        })
        .filter(|token| !token.is_empty())
        .map(|token| format!("\"{token}\""))
        .collect::<Vec<_>>()
        .join(" ")
}

fn binder_path(conn: &Connection, id: Uuid) -> AppResult<Vec<String>> {
    let mut path = Vec::new();
    let mut current = Some(id);
    while let Some(current_id) = current {
        let row: Option<(String, Option<String>)> = conn
            .query_row(
                "SELECT title, parent_id FROM binder_items WHERE id = ?1",
                params![current_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((title, parent_id)) = row else { break };
        path.push(title);
        current = parent_id
            .map(|id| {
                Uuid::parse_str(&id).map_err(|error| AppError::ProjectCorrupt(error.to_string()))
            })
            .transpose()?;
    }
    path.reverse();
    Ok(path)
}

fn keywords_from_json(value: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(value).unwrap_or_default()
}

fn sanitize_file_name(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        "export".to_string()
    } else {
        sanitized
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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

    use super::{export_project, search};
    use crate::binder;
    use crate::documents;
    use crate::domain::{
        BinderItemType, CreateBinderItemRequest, CreateProjectRequest, ExportFormat,
        ExportProjectRequest, ExportScope, ProjectType, SaveDocumentMetadataRequest,
        SaveDocumentRequest, SearchProjectRequest,
    };
    use crate::inspector;
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
                title: "Chapter 3".to_string(),
            },
        )
        .expect("create binder item");
        let document = documents::get(
            dir.path(),
            &crate::domain::GetDocumentRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id: item.id,
            },
        )
        .expect("get document");
        documents::save(
            dir.path(),
            &SaveDocumentRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id: item.id,
                content_json: json!({"type":"doc"}),
                content_plain_text: "Entraron en la casa abandonada antes del amanecer."
                    .to_string(),
                expected_revision: Some(document.revision),
            },
        )
        .expect("save document");
        (dir, item.id)
    }

    #[test]
    fn indexes_and_searches_project_text() {
        let (dir, id) = project_with_document();
        inspector::save_metadata(
            dir.path(),
            &SaveDocumentMetadataRequest {
                session_id: uuid::Uuid::new_v4(),
                binder_item_id: id,
                label: None,
                status: Some("Draft".to_string()),
                target_word_count: None,
                keywords: vec!["misterio".to_string()],
                custom_fields: json!({}),
                include_in_export: true,
            },
        )
        .expect("save metadata");

        let results = search(
            dir.path(),
            &SearchProjectRequest {
                session_id: uuid::Uuid::new_v4(),
                query: "casa abandonada".to_string(),
            },
        )
        .expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].binder_item_id, id);
        assert_eq!(results[0].path, vec!["Chapter 3"]);
        assert!(results[0].snippet.contains("casa"));
    }

    #[test]
    fn exports_txt_markdown_and_json() {
        let (dir, id) = project_with_document();
        let base = ExportProjectRequest {
            session_id: uuid::Uuid::new_v4(),
            scope: ExportScope::Document,
            format: ExportFormat::Txt,
            binder_item_id: Some(id),
            include_titles: true,
            separate_scenes: true,
        };

        let txt = export_project(dir.path(), &base).expect("txt export");
        assert!(txt.content.contains("Chapter 3"));
        assert!(txt.content.contains("casa abandonada"));

        let markdown = export_project(
            dir.path(),
            &ExportProjectRequest {
                format: ExportFormat::Markdown,
                ..base.clone()
            },
        )
        .expect("markdown export");
        assert!(markdown.content.starts_with("# Chapter 3"));

        let json = export_project(
            dir.path(),
            &ExportProjectRequest {
                format: ExportFormat::Json,
                ..base
            },
        )
        .expect("json export");
        assert!(json.content.contains("\"documents\""));
    }
}
