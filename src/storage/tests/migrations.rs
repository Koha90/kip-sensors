use super::super::{MIGRATIONS, SqliteSensorStore, StorageError, StorageResult};

use rusqlite::{Connection, params};

#[test]
fn open_creates_sensors_table() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let count: i64 = store.conn.query_row(
        "SELECT COUNT(*)
             FROM sqlite_master
             WHERE type = 'table'
               AND name = 'sensors'",
        [],
        |row| row.get(0),
    )?;

    assert_eq!(count, 1);

    Ok(())
}

#[test]
fn open_initializes_schema_version() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let version: i64 = store
        .conn
        .pragma_query_value(None, "user_version", |row| {
            row.get(0)
        })?;

    assert_eq!(version, 2);

    Ok(())
}

#[test]
fn unfinished_transaction_rolls_back() -> StorageResult<()> {
    let mut store = SqliteSensorStore::open(":memory:")?;

    {
        let tx = store.conn.transaction()?;

        tx.execute(
            "INSERT INTO sensors (id, name, location, status)
                 VALUES (?1, ?2, ?3, ?4)",
            params![1, "PT-101", "Пастеризатор", "Active"],
        )?;

        // No commit: the transaction is dropped here.
    }

    // The insered sensor must not exist after rollback.
    assert!(store.find(1)?.is_none());

    Ok(())
}

#[test]
fn open_rejects_unsupported_schema_version() -> StorageResult<()> {
    // Create an isolated temporary directory.
    let dir = tempfile::tempdir().expect("temporary directory must be created");

    // Build the database file path.
    let db_path = dir
        .path()
        .join("sensors_test.db");

    // Create a database with an unsuppoted schema version.
    let conn = Connection::open(&db_path)?;

    conn.pragma_update(None, "user_version", 99_i64)?;

    // Close the connection before reopening the database.
    drop(conn);

    // Attempt to open the database through our storage API.
    let result = SqliteSensorStore::open(&db_path);

    // Verify the exact error variant and its version.
    assert!(matches!(
        result,
        Err(StorageError::Migration(
            rusqlite_migration::Error::MigrationDefinition(
                rusqlite_migration::MigrationDefinitionError::DatabaseTooFarAhead
            )
        ))
    ));

    Ok(())
}

#[test]
fn opening_database_after_tempdir_drop_fails() {
    let dir = tempfile::tempdir().unwrap();

    let db_path = dir.path().join("sensors.db");

    drop(dir);

    let result = rusqlite::Connection::open(&db_path);

    assert!(result.is_err());
}

#[test]
fn migrations_are_valid() {
    MIGRATIONS
        .validate()
        .expect("all migrations must be valid");
}
