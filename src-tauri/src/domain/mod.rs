use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const APP_NAME: &str = "Local Writer";
pub const PROJECT_FORMAT: &str = "local-writer-project";
pub const PROJECT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProjectType {
    Blank,
    Novel,
    Screenplay,
}

impl ProjectType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Blank => "blank",
            Self::Novel => "novel",
            Self::Screenplay => "screenplay",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectJson {
    pub format: String,
    pub format_version: u32,
    pub app_version: String,
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub project_type: ProjectType,
    pub created_at: String,
    pub updated_at: String,
    pub last_opened_at: Option<String>,
    pub settings: Value,
    pub custom_metadata_schema: Vec<Value>,
    pub database: ProjectDatabaseJson,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDatabaseJson {
    pub file: String,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub project_type: ProjectType,
    pub path: String,
    pub format_version: u32,
    pub database_schema_version: u32,
    pub last_opened_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSession {
    pub session_id: Uuid,
    pub project: ProjectSummary,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub folder_path: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub project_type: ProjectType,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDefaultProjectRequest {
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub project_type: ProjectType,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenProjectRequest {
    pub folder_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseProjectRequest {
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseProjectResponse {
    pub closed: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BackupKind {
    Automatic,
    Manual,
}

impl BackupKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Manual => "manual",
        }
    }
}

impl TryFrom<&str> for BackupKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "automatic" => Ok(Self::Automatic),
            "manual" => Ok(Self::Manual),
            other => Err(format!("unknown backup kind {other}")),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBackupRequest {
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBackupsRequest {
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRecord {
    pub id: Uuid,
    pub created_at: String,
    pub path: String,
    pub kind: BackupKind,
    pub format_version: u32,
    pub size_bytes: Option<i64>,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockFile {
    pub app: String,
    pub app_version: String,
    pub pid: u32,
    pub host: Option<String>,
    pub created_at: String,
    pub heartbeat_at: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BinderItemType {
    Folder,
    Document,
    Research,
    Character,
    Location,
    Note,
    Trash,
}

impl BinderItemType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Folder => "folder",
            Self::Document => "document",
            Self::Research => "research",
            Self::Character => "character",
            Self::Location => "location",
            Self::Note => "note",
            Self::Trash => "trash",
        }
    }

    pub fn has_document(self) -> bool {
        !matches!(self, Self::Folder | Self::Trash)
    }
}

impl TryFrom<&str> for BinderItemType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "folder" => Ok(Self::Folder),
            "document" => Ok(Self::Document),
            "research" => Ok(Self::Research),
            "character" => Ok(Self::Character),
            "location" => Ok(Self::Location),
            "note" => Ok(Self::Note),
            "trash" => Ok(Self::Trash),
            other => Err(format!("unknown binder item type {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinderItem {
    pub id: Uuid,
    pub project_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub item_type: BinderItemType,
    pub title: String,
    pub synopsis: String,
    pub position: i64,
    pub icon: Option<String>,
    pub color_label: Option<String>,
    pub status: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_expanded: bool,
    pub is_archived: bool,
    pub trashed_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBinderItemsRequest {
    pub session_id: Uuid,
    #[serde(default)]
    pub include_trashed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBinderItemRequest {
    pub session_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub item_type: BinderItemType,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameBinderItemRequest {
    pub session_id: Uuid,
    pub item_id: Uuid,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBinderItemExpandedRequest {
    pub session_id: Uuid,
    pub item_id: Uuid,
    pub is_expanded: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateBinderItemRequest {
    pub session_id: Uuid,
    pub item_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveBinderItemRequest {
    pub session_id: Uuid,
    pub item_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub position: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderBinderItemsRequest {
    pub session_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub ordered_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashBinderItemRequest {
    pub session_id: Uuid,
    pub item_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreBinderItemRequest {
    pub session_id: Uuid,
    pub item_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRecord {
    pub id: Uuid,
    pub binder_item_id: Uuid,
    pub content_json: Value,
    pub content_plain_text: String,
    pub word_count: i64,
    pub character_count: i64,
    pub revision: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDocumentRequest {
    pub session_id: Uuid,
    pub binder_item_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDocumentRequest {
    pub session_id: Uuid,
    pub binder_item_id: Uuid,
    pub content_json: Value,
    pub content_plain_text: String,
    pub expected_revision: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDocumentRecoveryRequest {
    pub session_id: Uuid,
    pub binder_item_id: Uuid,
    pub content_json: Value,
    pub content_plain_text: String,
    pub revision: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDocumentRecoveryRequest {
    pub session_id: Uuid,
    pub binder_item_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRecoveryState {
    pub binder_item_id: Uuid,
    pub content_json: Value,
    pub content_plain_text: String,
    pub revision: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearDocumentRecoveryRequest {
    pub session_id: Uuid,
    pub binder_item_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMetadata {
    pub document_id: Uuid,
    pub label: Option<String>,
    pub status: Option<String>,
    pub target_word_count: Option<i64>,
    pub keywords: Vec<String>,
    pub custom_fields: Value,
    pub include_in_export: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectNote {
    pub id: Uuid,
    pub project_id: Uuid,
    pub binder_item_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub id: Uuid,
    pub document_id: Uuid,
    pub name: String,
    pub content_json: Value,
    pub content_plain_text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorData {
    pub metadata: Option<DocumentMetadata>,
    pub notes: Vec<ProjectNote>,
    pub snapshots: Vec<Snapshot>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInspectorDataRequest {
    pub session_id: Uuid,
    pub binder_item_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveBinderSynopsisRequest {
    pub session_id: Uuid,
    pub item_id: Uuid,
    pub synopsis: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDocumentMetadataRequest {
    pub session_id: Uuid,
    pub binder_item_id: Uuid,
    pub label: Option<String>,
    pub status: Option<String>,
    pub target_word_count: Option<i64>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub custom_fields: Value,
    pub include_in_export: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProjectNoteRequest {
    pub session_id: Uuid,
    pub id: Option<Uuid>,
    pub binder_item_id: Option<Uuid>,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSnapshotRequest {
    pub session_id: Uuid,
    pub binder_item_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreSnapshotRequest {
    pub session_id: Uuid,
    pub snapshot_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchProjectRequest {
    pub session_id: Uuid,
    pub query: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub binder_item_id: Uuid,
    pub title: String,
    pub item_type: BinderItemType,
    pub path: Vec<String>,
    pub snippet: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExportScope {
    Document,
    Folder,
    Included,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExportFormat {
    Txt,
    Markdown,
    Html,
    Json,
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Txt => "txt",
            Self::Markdown => "md",
            Self::Html => "html",
            Self::Json => "json",
        }
    }

    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Txt => "text/plain",
            Self::Markdown => "text/markdown",
            Self::Html => "text/html",
            Self::Json => "application/json",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProjectRequest {
    pub session_id: Uuid,
    pub scope: ExportScope,
    pub format: ExportFormat,
    pub binder_item_id: Option<Uuid>,
    #[serde(default = "default_true")]
    pub include_titles: bool,
    #[serde(default = "default_true")]
    pub separate_scenes: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedFile {
    pub file_name: String,
    pub mime_type: String,
    pub content: String,
}

fn default_true() -> bool {
    true
}
