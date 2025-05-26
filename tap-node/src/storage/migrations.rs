use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use super::error::StorageError;

const MIGRATIONS: &[&str] = &[
    include_str!("../../migrations/0001_create_transactions.sql"),
    include_str!("../../migrations/0002_create_messages.sql"),
];

pub fn run_migrations(conn: &mut Connection) -> Result<(), StorageError> {
    let migrations = Migrations::new(
        MIGRATIONS
            .iter()
            .map(|sql| M::up(sql).down(""))
            .collect::<Vec<_>>(),
    );

    migrations
        .to_latest(conn)
        .map_err(|e| StorageError::Migration(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_migrations() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let mut conn = Connection::open(&db_path).unwrap();

        assert!(run_migrations(&mut conn).is_ok());

        // Verify tables exist
        let table_exists: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='transactions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_exists, 1);
    }
}
