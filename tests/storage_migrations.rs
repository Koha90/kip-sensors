/// Intgration tests for database schema upgrades.
use kip_sensors::sensor::SensorStatus;
use kip_sensors::storage::{SqliteSensorStore, StorageResult};
use rusqlite::{Connection, params};

/// Verifies upgrading a version-1 database to version 2
/// without losing existing sensor records.
#[test]
fn updgrade_v1_to_v2_preserves_sensor() -> StorageResult<()> {
    let dir = tempfile::tempdir().expect("temporary directory must be created");

    let db_path = dir
        .path()
        .join("old_sensors.db");

    // Arrange: create a database using only the v1 schema.
    {
        let conn = Connection::open(&db_path)?;

        conn.execute_batch(include_str!(
            "../migrations/001_create_sensors.sql"
        ))?;

        conn.execute(
            "INSERT INTO sensors (id, name, location, status)
             VALUES (?1, ?2, ?3, ?4)",
            params![1, "PT-101", "Пастеризатор", "Maintenance"],
        )?;

        conn.pragma_update(None, "user_version", 1_i64)?;
    }

    // Act: opening the database applies pending migrations.
    let store = SqliteSensorStore::open(&db_path)?;

    // Assert: the existing sensor and its status are preserved.
    let sensor = store
        .find(1)?
        .expect("existing sensor must survive migration");

    assert_eq!(sensor.id(), 1);
    assert_eq!(sensor.name(), "PT-101");
    assert_eq!(sensor.location(), "Пастеризатор");
    assert_eq!(
        sensor.status(),
        SensorStatus::Maintenance
    );

    drop(store);

    // Assert: schema version and new structures are present.
    let conn = Connection::open(&db_path)?;

    let version: i64 = conn.pragma_query_value(None, "user_version", |row| {
        row.get(0)
    })?;

    assert_eq!(version, 2);

    let model_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sensor_models",
        [],
        |row| row.get(0),
    )?;

    assert_eq!(model_count, 0);

    let model_id: Option<i64> = conn.query_row(
        "SELECT model_id FROM sensors WHERE id = ?1",
        [1],
        |row| row.get(0),
    )?;

    assert_eq!(model_id, None);

    Ok(())
}
