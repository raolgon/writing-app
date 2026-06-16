use std::path::Path;

use rusqlite::Connection;

use crate::errors::AppResult;
use crate::migrations;

pub const CURRENT_SCHEMA_VERSION: u32 = 2;

pub fn open_project_database(path: &Path) -> AppResult<Connection> {
    let mut conn = Connection::open(path)?;
    migrations::migrate(&mut conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::open_project_database;
    use super::CURRENT_SCHEMA_VERSION;

    #[test]
    fn starts_with_schema_version_two() {
        assert_eq!(CURRENT_SCHEMA_VERSION, 2);
    }

    #[test]
    fn opens_and_migrates_project_database() {
        let dir = tempdir().expect("tempdir");
        let db = dir.path().join("project.db");
        let conn = open_project_database(&db).expect("open db");

        let user_version: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("user_version");

        assert_eq!(user_version, CURRENT_SCHEMA_VERSION);
    }
}
