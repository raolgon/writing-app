# Project Format

## Goals

The project format is designed to be:

- Local-first.
- Portable between machines.
- Understandable without the application.
- Durable under crashes.
- Compatible with future migrations.
- Suitable for optional encryption in a later version.

## Folder layout

Each writing project lives in a user-selected folder:

```text
project-name/
  project.json
  project.db
  assets/
  backups/
  exports/
```

### `project.json`

Human-readable project metadata and format version marker.

### `project.db`

SQLite database containing structured project content.

### `assets/`

User-managed attachments and future embedded media. The MVP should preserve files but does not need advanced asset management.

### `backups/`

Local project backups and backup manifest files.

### `exports/`

Default export destination. Users may choose another destination during export.

## Format version

Initial format version:

```text
1
```

The application must validate the format version before opening a project for writing.

Rules:

- Same major version: open and run migrations if needed.
- Higher unsupported version: refuse to open for writing and show a clear message.
- Missing version: treat as invalid unless a future import flow supports it.

## `project.json` schema

Initial shape:

```json
{
  "format": "local-writer-project",
  "formatVersion": 1,
  "appVersion": "0.1.0",
  "id": "uuid",
  "title": "Project title",
  "description": "",
  "projectType": "blank",
  "createdAt": "2026-06-15T00:00:00.000Z",
  "updatedAt": "2026-06-15T00:00:00.000Z",
  "lastOpenedAt": "2026-06-15T00:00:00.000Z",
  "settings": {
    "backupRetentionCount": 10,
    "editor": {
      "textWidth": "medium",
      "fontSize": 18,
      "lineHeight": 1.7
    }
  },
  "customMetadataSchema": [],
  "database": {
    "file": "project.db",
    "schemaVersion": 1
  }
}
```

The exact TypeScript and Rust schemas will be created in Phase 2/3. Zod validation should reject unknown critical format identifiers and invalid date or version values.

## Atomic `project.json` writes

When updating `project.json`:

1. Serialize canonical JSON.
2. Write to a temporary file in the same folder.
3. Flush the file.
4. Rename the temporary file over `project.json`.
5. Flush the parent directory where supported.

This prevents partial metadata writes after a crash.

## SQLite schema

The first database schema should include the tables below. Names are intentionally plain and stable.

### `projects`

Mirrors the canonical project row and supports database-level consistency checks.

| Column                        | Type                     | Notes                                      |
| ----------------------------- | ------------------------ | ------------------------------------------ |
| `id`                          | text primary key         | UUID.                                      |
| `title`                       | text not null            | Project title.                             |
| `description`                 | text not null default '' | Project description.                       |
| `project_type`                | text not null            | `blank`, `novel`, `screenplay`, or custom. |
| `created_at`                  | text not null            | ISO timestamp.                             |
| `updated_at`                  | text not null            | ISO timestamp.                             |
| `last_opened_at`              | text                     | ISO timestamp.                             |
| `settings_json`               | text not null            | JSON object.                               |
| `custom_metadata_schema_json` | text not null            | JSON array.                                |
| `format_version`              | integer not null         | Project format version.                    |

### `binder_items`

| Column        | Type                       | Notes                                                                       |
| ------------- | -------------------------- | --------------------------------------------------------------------------- |
| `id`          | text primary key           | UUID.                                                                       |
| `project_id`  | text not null              | FK to `projects.id`.                                                        |
| `parent_id`   | text                       | FK to `binder_items.id`.                                                    |
| `type`        | text not null              | `folder`, `document`, `research`, `character`, `location`, `note`, `trash`. |
| `title`       | text not null              | Display title.                                                              |
| `synopsis`    | text not null default ''   | Short summary.                                                              |
| `position`    | integer not null           | Ordering among siblings.                                                    |
| `icon`        | text                       | Optional icon key.                                                          |
| `color_label` | text                       | Optional label key.                                                         |
| `status`      | text                       | Optional status key.                                                        |
| `created_at`  | text not null              | ISO timestamp.                                                              |
| `updated_at`  | text not null              | ISO timestamp.                                                              |
| `is_expanded` | integer not null default 0 | Boolean.                                                                    |
| `is_archived` | integer not null default 0 | Boolean.                                                                    |
| `trashed_at`  | text                       | Null unless moved to trash.                                                 |

Normal deletion moves an item into trash by setting `trashed_at` and/or changing parent to a trash container. Permanent purge is out of MVP scope unless implemented as a clearly separate explicit action.

### `documents`

| Column               | Type                       | Notes                     |
| -------------------- | -------------------------- | ------------------------- |
| `id`                 | text primary key           | UUID.                     |
| `binder_item_id`     | text not null unique       | FK to `binder_items.id`.  |
| `content_json`       | text not null              | Canonical TipTap JSON.    |
| `content_plain_text` | text not null              | Derived text.             |
| `word_count`         | integer not null default 0 | Derived count.            |
| `character_count`    | integer not null default 0 | Derived count.            |
| `revision`           | integer not null default 0 | Incremented on each save. |
| `created_at`         | text not null              | ISO timestamp.            |
| `updated_at`         | text not null              | ISO timestamp.            |

### `document_metadata`

| Column               | Type                       | Notes                  |
| -------------------- | -------------------------- | ---------------------- |
| `document_id`        | text primary key           | FK to `documents.id`.  |
| `label`              | text                       | User label.            |
| `status`             | text                       | Workflow status.       |
| `target_word_count`  | integer                    | Optional target.       |
| `keywords_json`      | text not null              | JSON array of strings. |
| `custom_fields_json` | text not null              | JSON object.           |
| `include_in_export`  | integer not null default 1 | Boolean.               |

### `project_notes`

| Column           | Type             | Notes                                         |
| ---------------- | ---------------- | --------------------------------------------- |
| `id`             | text primary key | UUID.                                         |
| `project_id`     | text not null    | FK to `projects.id`.                          |
| `binder_item_id` | text             | Optional FK.                                  |
| `title`          | text not null    | Note title.                                   |
| `content`        | text not null    | Plain text or future structured note content. |
| `created_at`     | text not null    | ISO timestamp.                                |
| `updated_at`     | text not null    | ISO timestamp.                                |

### `snapshots`

| Column               | Type             | Notes                         |
| -------------------- | ---------------- | ----------------------------- |
| `id`                 | text primary key | UUID.                         |
| `document_id`        | text not null    | FK to `documents.id`.         |
| `name`               | text not null    | User-visible snapshot name.   |
| `content_json`       | text not null    | TipTap JSON at snapshot time. |
| `content_plain_text` | text not null    | Plain text at snapshot time.  |
| `created_at`         | text not null    | ISO timestamp.                |

### `session_state`

| Column       | Type             | Notes          |
| ------------ | ---------------- | -------------- |
| `key`        | text primary key | Stable key.    |
| `value_json` | text not null    | JSON value.    |
| `updated_at` | text not null    | ISO timestamp. |

Stores last active document, active view, recovery flags, and similar project-local session data.

### `backup_records`

| Column           | Type             | Notes                           |
| ---------------- | ---------------- | ------------------------------- |
| `id`             | text primary key | UUID.                           |
| `created_at`     | text not null    | ISO timestamp.                  |
| `path`           | text not null    | Relative path under `backups/`. |
| `kind`           | text not null    | `automatic` or `manual`.        |
| `format_version` | integer not null | Project format version.         |
| `size_bytes`     | integer          | Optional.                       |
| `status`         | text not null    | `complete` or `failed`.         |

## FTS5 search

The database should include an FTS5 virtual table, for example:

```sql
CREATE VIRTUAL TABLE search_index USING fts5(
  entity_type,
  entity_id,
  title,
  path,
  body,
  keywords,
  tokenize = 'unicode61'
);
```

The exact FTS strategy may change during implementation. The important contract is that global search covers titles, synopses, document text, notes, and keywords.

## Migrations

Migrations are Rust-owned and applied when opening a project.

Rules:

- Migrations are numbered and deterministic.
- Each migration runs in a transaction.
- Failed migrations leave the database at the previous valid schema version.
- The app must not silently downgrade or rewrite a newer unsupported schema.
- Migration tests must create older schema fixtures and verify upgraded state.

## Templates

Templates create initial binder structures only. They do not restrict later user organization.

### Blank

```text
Project root
```

### Novel

```text
Manuscript
Characters
Locations
Research
Notes
```

### Screenplay

```text
Script
Characters
Locations
Research
Notes
```

The screenplay template uses normal documents in the MVP. The model should later support Fountain-compatible screenplay documents without changing the project folder contract.

## Lock file

Writable project sessions use a lock file such as:

```text
.write-lock
```

Initial contents:

```json
{
  "app": "local-writer",
  "appVersion": "0.1.0",
  "pid": 12345,
  "host": "machine-name",
  "createdAt": "2026-06-15T00:00:00.000Z",
  "heartbeatAt": "2026-06-15T00:00:00.000Z"
}
```

The app must refuse concurrent write access unless the user explicitly handles a stale lock.

## Backup format

Initial backups may be stored as directories or archives. The simpler first implementation is a timestamped directory:

```text
backups/
  2026-06-15T120000Z/
    project.json
    project.db
    assets/
    backup.json
```

`backup.json` contains:

```json
{
  "id": "uuid",
  "createdAt": "2026-06-15T12:00:00.000Z",
  "kind": "automatic",
  "formatVersion": 1,
  "databaseSchemaVersion": 1,
  "appVersion": "0.1.0"
}
```

Compression can be added later after correctness and recovery are tested.

## Export format

Initial exports go to `exports/` by default and support:

- `.txt`
- `.md`
- `.html`
- `.json`

Export options:

- Scope: document, folder, or all included documents.
- Include titles.
- Scene separator.
- Binder order.
- Include or exclude archived items.

The full project JSON export is not the live storage format. It is an interchange/export artifact assembled from `project.json` and `project.db`.

## Future encryption compatibility

Encryption is not part of the MVP. To keep the option open:

- Keep filesystem and database access behind Rust services.
- Avoid direct frontend reads from project files.
- Keep assets under the project folder so they can later be encrypted or packaged consistently.
- Keep project format metadata explicit enough to mark encrypted projects later.
