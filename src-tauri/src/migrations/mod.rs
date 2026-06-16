use rusqlite::{Connection, OptionalExtension};

use crate::database::CURRENT_SCHEMA_VERSION;
use crate::errors::AppResult;

#[derive(Debug, Clone, Copy)]
pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const INITIAL_MIGRATION: Migration = Migration {
    version: 1,
    name: "initial_schema",
    sql: r#"
CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY NOT NULL,
  title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  project_type TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  last_opened_at TEXT,
  settings_json TEXT NOT NULL,
  custom_metadata_schema_json TEXT NOT NULL,
  format_version INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS binder_items (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  parent_id TEXT,
  type TEXT NOT NULL CHECK (type IN ('folder', 'document', 'research', 'character', 'location', 'note', 'trash')),
  title TEXT NOT NULL,
  synopsis TEXT NOT NULL DEFAULT '',
  position INTEGER NOT NULL,
  icon TEXT,
  color_label TEXT,
  status TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  is_expanded INTEGER NOT NULL DEFAULT 0,
  is_archived INTEGER NOT NULL DEFAULT 0,
  trashed_at TEXT,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  FOREIGN KEY (parent_id) REFERENCES binder_items(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_binder_items_project_parent_position
  ON binder_items(project_id, parent_id, position);

CREATE TABLE IF NOT EXISTS documents (
  id TEXT PRIMARY KEY NOT NULL,
  binder_item_id TEXT NOT NULL UNIQUE,
  content_json TEXT NOT NULL,
  content_plain_text TEXT NOT NULL,
  word_count INTEGER NOT NULL DEFAULT 0,
  character_count INTEGER NOT NULL DEFAULT 0,
  revision INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (binder_item_id) REFERENCES binder_items(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS document_metadata (
  document_id TEXT PRIMARY KEY NOT NULL,
  label TEXT,
  status TEXT,
  target_word_count INTEGER,
  keywords_json TEXT NOT NULL,
  custom_fields_json TEXT NOT NULL,
  include_in_export INTEGER NOT NULL DEFAULT 1,
  FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS project_notes (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL,
  binder_item_id TEXT,
  title TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  FOREIGN KEY (binder_item_id) REFERENCES binder_items(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS snapshots (
  id TEXT PRIMARY KEY NOT NULL,
  document_id TEXT NOT NULL,
  name TEXT NOT NULL,
  content_json TEXT NOT NULL,
  content_plain_text TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS session_state (
  key TEXT PRIMARY KEY NOT NULL,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS backup_records (
  id TEXT PRIMARY KEY NOT NULL,
  created_at TEXT NOT NULL,
  path TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('automatic', 'manual')),
  format_version INTEGER NOT NULL,
  size_bytes INTEGER,
  status TEXT NOT NULL CHECK (status IN ('complete', 'failed'))
);
"#,
};

pub const SEARCH_EXPORT_MIGRATION: Migration = Migration {
    version: 2,
    name: "search_index",
    sql: r#"
CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
  binder_item_id UNINDEXED,
  title,
  synopsis,
  content,
  notes,
  keywords
);
"#,
};

const MIGRATIONS: &[Migration] = &[INITIAL_MIGRATION, SEARCH_EXPORT_MIGRATION];

pub fn migrate(conn: &mut Connection) -> AppResult<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )?;

    let tx = conn.transaction()?;
    for migration in MIGRATIONS {
        let already_applied = tx
            .query_row(
                "SELECT version FROM schema_migrations WHERE version = ?1",
                [migration.version],
                |row| row.get::<_, u32>(0),
            )
            .optional()?
            .is_some();

        if !already_applied {
            tx.execute_batch(migration.sql)?;
            tx.execute(
                "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, datetime('now'))",
                (migration.version, migration.name),
            )?;
        }
    }

    tx.pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION)?;
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{migrate, INITIAL_MIGRATION, SEARCH_EXPORT_MIGRATION};

    #[test]
    fn initial_migration_matches_schema_version() {
        assert_eq!(INITIAL_MIGRATION.version, 1);
        assert_eq!(SEARCH_EXPORT_MIGRATION.version, 2);
    }

    #[test]
    fn applies_initial_schema() {
        let mut conn = Connection::open_in_memory().expect("open in-memory db");

        migrate(&mut conn).expect("migrate");

        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version IN (1, 2)",
                [],
                |row| row.get(0),
            )
            .expect("migration row");

        assert_eq!(count, 2);
    }
}
