use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use uuid::Uuid;

use crate::backups;
use crate::binder;
use crate::database;
use crate::documents;
use crate::domain::{
    self, BackupRecord, BinderItem, ClearDocumentRecoveryRequest, CloseProjectRequest,
    CloseProjectResponse, CreateBackupRequest, CreateBinderItemRequest,
    CreateDefaultProjectRequest, CreateProjectRequest, CreateSnapshotRequest, DocumentMetadata,
    DocumentRecord, DocumentRecoveryState, DuplicateBinderItemRequest, ExportProjectRequest,
    ExportedFile, GetDocumentRecoveryRequest, GetDocumentRequest, GetInspectorDataRequest,
    InspectorData, ListBackupsRequest, ListBinderItemsRequest, MoveBinderItemRequest,
    OpenProjectRequest, ProjectNote, ProjectSession, RecordDocumentRecoveryRequest,
    RenameBinderItemRequest, ReorderBinderItemsRequest, RestoreBinderItemRequest,
    RestoreSnapshotRequest, SaveBinderSynopsisRequest, SaveDocumentMetadataRequest,
    SaveDocumentRequest, SaveProjectNoteRequest, SearchProjectRequest, SearchResult,
    SetBinderItemExpandedRequest, Snapshot, TrashBinderItemRequest,
};
use crate::errors::{AppError, CommandError};
use crate::inspector;
use crate::locks::ProjectLock;
use crate::project;
use crate::search_export;

#[derive(Debug, Default)]
pub struct CommandState {
    open_projects: Mutex<HashMap<Uuid, OpenProject>>,
}

#[derive(Debug)]
struct OpenProject {
    #[allow(dead_code)]
    path: PathBuf,
    lock: ProjectLock,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub project_format: &'static str,
    pub project_format_version: u32,
    pub database_schema_version: u32,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: domain::APP_NAME,
        version: env!("CARGO_PKG_VERSION"),
        project_format: domain::PROJECT_FORMAT,
        project_format_version: domain::PROJECT_FORMAT_VERSION,
        database_schema_version: database::CURRENT_SCHEMA_VERSION,
    }
}

#[tauri::command]
pub fn create_project(
    state: tauri::State<'_, CommandState>,
    request: CreateProjectRequest,
) -> Result<ProjectSession, CommandError> {
    let summary = project::create_project(&request).map_err(CommandError::from)?;
    let project_path = PathBuf::from(&summary.path);
    let lock = ProjectLock::acquire(&project_path).map_err(CommandError::from)?;
    let session_id = Uuid::new_v4();
    state
        .open_projects
        .lock()
        .map_err(|_| CommandError::from(AppError::Validation("state lock poisoned".to_string())))?
        .insert(
            session_id,
            OpenProject {
                path: project_path,
                lock,
            },
        );

    Ok(ProjectSession {
        session_id,
        project: summary,
    })
}

#[tauri::command]
pub fn create_default_project(
    state: tauri::State<'_, CommandState>,
    request: CreateDefaultProjectRequest,
) -> Result<ProjectSession, CommandError> {
    let summary = project::create_default_project(&request).map_err(CommandError::from)?;
    let project_path = PathBuf::from(&summary.path);
    let lock = ProjectLock::acquire(&project_path).map_err(CommandError::from)?;
    let session_id = Uuid::new_v4();
    state
        .open_projects
        .lock()
        .map_err(|_| CommandError::from(AppError::Validation("state lock poisoned".to_string())))?
        .insert(
            session_id,
            OpenProject {
                path: project_path,
                lock,
            },
        );

    Ok(ProjectSession {
        session_id,
        project: summary,
    })
}

#[tauri::command]
pub fn open_project(
    state: tauri::State<'_, CommandState>,
    request: OpenProjectRequest,
) -> Result<ProjectSession, CommandError> {
    let project_path = PathBuf::from(&request.folder_path);
    if !project_path
        .join(crate::filesystem::PROJECT_JSON_FILE)
        .exists()
    {
        return Err(CommandError::from(AppError::ProjectNotFound(
            project_path.display().to_string(),
        )));
    }

    let lock = ProjectLock::acquire(&project_path).map_err(CommandError::from)?;
    let summary = match project::open_project(&project_path) {
        Ok(summary) => summary,
        Err(error) => {
            drop(lock);
            return Err(CommandError::from(error));
        }
    };
    let session_id = Uuid::new_v4();
    state
        .open_projects
        .lock()
        .map_err(|_| CommandError::from(AppError::Validation("state lock poisoned".to_string())))?
        .insert(
            session_id,
            OpenProject {
                path: project_path,
                lock,
            },
        );

    Ok(ProjectSession {
        session_id,
        project: summary,
    })
}

#[tauri::command]
pub fn close_project(
    state: tauri::State<'_, CommandState>,
    request: CloseProjectRequest,
) -> Result<CloseProjectResponse, CommandError> {
    let open_project = state
        .open_projects
        .lock()
        .map_err(|_| CommandError::from(AppError::Validation("state lock poisoned".to_string())))?
        .remove(&request.session_id)
        .ok_or_else(|| AppError::SessionNotFound(request.session_id.to_string()))
        .map_err(CommandError::from)?;

    let _ = backups::create_automatic_backup(&open_project.path);
    open_project.lock.release().map_err(CommandError::from)?;
    Ok(CloseProjectResponse { closed: true })
}

#[tauri::command]
pub fn create_backup(
    state: tauri::State<'_, CommandState>,
    request: CreateBackupRequest,
) -> Result<BackupRecord, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    backups::create_manual_backup(&project_path).map_err(CommandError::from)
}

#[tauri::command]
pub fn list_backups(
    state: tauri::State<'_, CommandState>,
    request: ListBackupsRequest,
) -> Result<Vec<BackupRecord>, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    backups::list_backups(&project_path).map_err(CommandError::from)
}

#[tauri::command]
pub fn list_binder_items(
    state: tauri::State<'_, CommandState>,
    request: ListBinderItemsRequest,
) -> Result<Vec<BinderItem>, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    binder::list(&project_path, request.include_trashed).map_err(CommandError::from)
}

#[tauri::command]
pub fn create_binder_item(
    state: tauri::State<'_, CommandState>,
    request: CreateBinderItemRequest,
) -> Result<BinderItem, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    binder::create(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn rename_binder_item(
    state: tauri::State<'_, CommandState>,
    request: RenameBinderItemRequest,
) -> Result<BinderItem, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    binder::rename(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn set_binder_item_expanded(
    state: tauri::State<'_, CommandState>,
    request: SetBinderItemExpandedRequest,
) -> Result<BinderItem, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    binder::set_expanded(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn duplicate_binder_item(
    state: tauri::State<'_, CommandState>,
    request: DuplicateBinderItemRequest,
) -> Result<BinderItem, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    binder::duplicate(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn move_binder_item(
    state: tauri::State<'_, CommandState>,
    request: MoveBinderItemRequest,
) -> Result<Vec<BinderItem>, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    binder::move_item(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn reorder_binder_items(
    state: tauri::State<'_, CommandState>,
    request: ReorderBinderItemsRequest,
) -> Result<Vec<BinderItem>, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    binder::reorder(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn trash_binder_item(
    state: tauri::State<'_, CommandState>,
    request: TrashBinderItemRequest,
) -> Result<Vec<BinderItem>, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    binder::trash(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn restore_binder_item(
    state: tauri::State<'_, CommandState>,
    request: RestoreBinderItemRequest,
) -> Result<Vec<BinderItem>, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    binder::restore(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn get_document(
    state: tauri::State<'_, CommandState>,
    request: GetDocumentRequest,
) -> Result<DocumentRecord, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    documents::get(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn save_document(
    state: tauri::State<'_, CommandState>,
    request: SaveDocumentRequest,
) -> Result<DocumentRecord, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    documents::save(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn record_document_recovery(
    state: tauri::State<'_, CommandState>,
    request: RecordDocumentRecoveryRequest,
) -> Result<DocumentRecoveryState, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    documents::record_recovery(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn get_document_recovery(
    state: tauri::State<'_, CommandState>,
    request: GetDocumentRecoveryRequest,
) -> Result<Option<DocumentRecoveryState>, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    documents::get_recovery(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn clear_document_recovery(
    state: tauri::State<'_, CommandState>,
    request: ClearDocumentRecoveryRequest,
) -> Result<(), CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    documents::clear_recovery(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn get_inspector_data(
    state: tauri::State<'_, CommandState>,
    request: GetInspectorDataRequest,
) -> Result<InspectorData, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    inspector::get_data(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn save_binder_synopsis(
    state: tauri::State<'_, CommandState>,
    request: SaveBinderSynopsisRequest,
) -> Result<BinderItem, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    inspector::save_synopsis(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn save_document_metadata(
    state: tauri::State<'_, CommandState>,
    request: SaveDocumentMetadataRequest,
) -> Result<DocumentMetadata, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    inspector::save_metadata(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn save_project_note(
    state: tauri::State<'_, CommandState>,
    request: SaveProjectNoteRequest,
) -> Result<ProjectNote, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    inspector::save_note(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn create_snapshot(
    state: tauri::State<'_, CommandState>,
    request: CreateSnapshotRequest,
) -> Result<Snapshot, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    inspector::create_snapshot(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn restore_snapshot(
    state: tauri::State<'_, CommandState>,
    request: RestoreSnapshotRequest,
) -> Result<DocumentRecord, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    inspector::restore_snapshot(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn search_project(
    state: tauri::State<'_, CommandState>,
    request: SearchProjectRequest,
) -> Result<Vec<SearchResult>, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    search_export::search(&project_path, &request).map_err(CommandError::from)
}

#[tauri::command]
pub fn export_project(
    state: tauri::State<'_, CommandState>,
    request: ExportProjectRequest,
) -> Result<ExportedFile, CommandError> {
    let project_path = project_path_for_session(&state, request.session_id)?;
    search_export::export_project(&project_path, &request).map_err(CommandError::from)
}

fn project_path_for_session(
    state: &tauri::State<'_, CommandState>,
    session_id: Uuid,
) -> Result<PathBuf, CommandError> {
    state
        .open_projects
        .lock()
        .map_err(|_| CommandError::from(AppError::Validation("state lock poisoned".to_string())))?
        .get(&session_id)
        .map(|project| project.path.clone())
        .ok_or_else(|| CommandError::from(AppError::SessionNotFound(session_id.to_string())))
}
