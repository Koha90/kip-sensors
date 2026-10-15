use super::super::{SqliteSensorStore, StorageError, StorageResult};

use crate::sensor::{Sensor, SensorStatus};
use crate::sensor_model::SensorModel;

use rusqlite::params;

#[test]
fn insert_sensor_success() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let mut sensor = Sensor::new(
        1,
        "PT-101".to_string(),
        "Пастеризатор".to_string(),
    )
    .expect("sensor must be created");

    sensor.maintenance();

    store.insert(&sensor)?;

    let (name, status): (String, String) = store.conn.query_row(
        "SELECT name, status FROM sensors WHERE id = ?1",
        [1],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    assert_eq!(name, "PT-101");
    assert_eq!(status, "Maintenance");

    Ok(())
}

#[test]
fn find_existing_sensor() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let mut sensor = Sensor::new(
        1,
        "PT-101".to_string(),
        "Пастеризатор".to_string(),
    )
    .expect("sensor must be created");

    sensor.maintenance();

    store.insert(&sensor)?;

    let restored = store
        .find(1)?
        .expect("sensor must be found");

    assert_eq!(restored.id(), 1);
    assert_eq!(restored.name(), "PT-101");
    assert_eq!(restored.location(), "Пастеризатор");
    assert_eq!(
        restored.status(),
        SensorStatus::Maintenance
    );

    Ok(())
}

#[test]
fn search_matches_name_and_location() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let mut sensors = [
        Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор, линия 1".to_string(),
        )?,
        Sensor::new(
            2,
            "PT-202".to_string(),
            "Танк хранения".to_string(),
        )?,
        Sensor::new(
            3,
            "TT-303".to_string(),
            "Пастеризатор, линия 2".to_string(),
        )?,
    ];

    let model = SensorModel::new(
        7,
        "WIKA".to_string(),
        "A-10".to_string(),
        Some("A-10-123".to_string()),
    )?;

    store.insert_model(&model)?;

    sensors[0].assign_model(model.id());

    // Arrange: persist all test sensors.
    for sensor in &sensors {
        store.insert(sensor)?;
    }

    // Act: search by installation location.
    let by_location = store.search("Пастеризатор")?;

    assert_eq!(by_location.len(), 2);
    assert_eq!(by_location[0].id(), 1);
    assert_eq!(by_location[1].id(), 3);

    assert_eq!(by_location[0].model_id(), Some(7));
    assert_eq!(by_location[1].model_id(), None);

    // Act: search by sensor name.
    let by_name = store.search("PT")?;

    assert_eq!(by_name.len(), 2);
    assert_eq!(by_name[0].id(), 1);
    assert_eq!(by_name[1].id(), 2);

    assert_eq!(by_name[0].model_id(), Some(7));
    assert_eq!(by_name[1].model_id(), None);
    Ok(())
}

#[test]
fn sensor_cannot_reference_missing_model() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let sensor = Sensor::new(
        1,
        "PT-101".to_string(),
        "Пастеризатор".to_string(),
    )?;

    store.insert(&sensor)?;

    // Try to reference a model that does not exist.
    let result = store.conn.execute(
        "UPDATE sensors SET model_id = ?1 WHERE id = ?2",
        params![999, 1],
    );

    assert!(matches!(
            result,
            Err(rusqlite::Error::SqliteFailure(err, _))
            if err.extended_code
            == rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY
    ));

    // Verify that the rejected UPDATE did not change the record.
    let model_id: Option<u32> = store.conn.query_row(
        "SELECT model_id FROM sensors WHERE id = ?1",
        [1],
        |row| row.get(0),
    )?;

    assert_eq!(model_id, None);

    Ok(())
}

#[test]
fn insert_duplicate_id_returns_error() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let sensor = Sensor::new(
        1,
        "PT-101".to_string(),
        "Пастеризатор".to_string(),
    )
    .expect("sensor must be created");

    // The first insertion must succeed.
    store.insert(&sensor)?;

    // The second insertion must violate the primary key.
    let result = store.insert(&sensor);

    assert!(matches!(
            result,
            Err(StorageError::Database(
                    rusqlite::Error::SqliteFailure(err, _)
            ))
                if err.extended_code
                    == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY
    ));

    Ok(())
}

#[test]
fn insert_sensor_with_model() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    // Arrange: create and persist a sensor model.
    let model = SensorModel::new(
        7,
        "WIKA".to_string(),
        "A-10".to_string(),
        None,
    )
    .expect("sensor model must be created");

    store.insert_model(&model)?;

    // Arrange: assign the existing model to a sensor.
    let mut sensor = Sensor::new(
        1,
        "PT-101".to_string(),
        "Пастеризатор".to_string(),
    )?;

    sensor.assign_model(model.id());

    // Act: persist the sensor.
    store.insert(&sensor)?;

    // Assert: the relationship is stored in SQLite.
    let saved_model_id: Option<u32> = store.conn.query_row(
        "SELECT model_id FROM sensors WHERE id = ?1",
        [1],
        |row| row.get(0),
    )?;

    assert_eq!(saved_model_id, Some(7));

    Ok(())
}

#[test]
fn insert_sensor_with_missing_model_returns_error() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let mut sensor = Sensor::new(
        1,
        "PT-101".to_string(),
        "Пастеризатор".to_string(),
    )?;

    sensor.assign_model(999);

    let result = store.insert(&sensor);

    assert!(matches!(
        result,
        Err(StorageError::Database(
            rusqlite::Error::SqliteFailure(err, _)
        )) if err.extended_code
        == rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY
    ));

    Ok(())
}

#[test]
fn list_empty_database_returns_empty_vec() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let sensors = store.list()?;

    assert!(sensors.is_empty());

    Ok(())
}

#[test]
fn delete_sensor_by_its_id_succeed() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let sensor = Sensor::new(
        1,
        "PT-101".to_string(),
        "Пастеризатор".to_string(),
    )?;

    // Arrange: persist a sensor before attempting deletion.
    store.insert(&sensor)?;

    // Act: delete the stored sensor.
    let removed = store.remove(1)?;

    // Assert: deletion succeeded and the record is gone.
    assert!(removed);
    assert!(store.find(1)?.is_none());

    Ok(())
}

#[test]
fn find_missing_sensor_returns_none() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let result = store.find(42)?;

    assert!(result.is_none());

    Ok(())
}

#[test]
fn search_returns_empty_for_unknown_query() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let sensor = Sensor::new(
        1,
        "PT-101".to_string(),
        "Пастеризатор".to_string(),
    )?;

    store.insert(&sensor)?;

    assert!(
        store
            .search("Несуществующий")?
            .is_empty()
    );
    assert!(store.search("")?.is_empty());

    // A percent sign is searched literally, not as a LIKE wildcard.
    assert!(store.search("%")?.is_empty());

    Ok(())
}

#[test]
fn remove_missing_sensor_returns_false() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let removed = store.remove(42)?;

    assert!(!removed);

    Ok(())
}

#[test]
fn find_sensor_restores_model_id() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let model = SensorModel::new(
        7,
        "WIKA".to_string(),
        "A-10".to_string(),
        None,
    )?;

    store.insert_model(&model)?;

    let mut sensor = Sensor::new(
        1,
        "PT-101".to_string(),
        "Пастеризатор".to_string(),
    )?;

    sensor.assign_model(model.id());
    store.insert(&sensor)?;

    let restored = store
        .find(1)?
        .expect("sensors must be found");

    assert_eq!(restored.model_id(), Some(7));

    Ok(())
}

#[test]
fn list_returns_sensors_ordered_by_id() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let mut first = Sensor::new(
        1,
        "PT-101".to_string(),
        "Пастеризатор".to_string(),
    )?;

    let second = Sensor::new(
        2,
        "PT-202".to_string(),
        "Танк хранения".to_string(),
    )?;

    first.maintenance();

    let model = SensorModel::new(
        7,
        "WIKA".to_string(),
        "A-10".to_string(),
        Some("A-10-123".to_string()),
    )?;

    store.insert_model(&model)?;

    first.assign_model(model.id());

    // Insert in reverse order to verify SQL sorting.
    store.insert(&second)?;
    store.insert(&first)?;

    let sensors = store.list()?;

    assert_eq!(sensors.len(), 2);

    assert_eq!(sensors[0].id(), 1);
    assert_eq!(sensors[0].name(), "PT-101");
    assert_eq!(
        sensors[0].status(),
        SensorStatus::Maintenance
    );

    assert_eq!(sensors[1].id(), 2);
    assert_eq!(sensors[1].name(), "PT-202");
    assert_eq!(
        sensors[1].status(),
        SensorStatus::Active
    );
    assert_eq!(sensors[0].model_id(), Some(7));
    assert_eq!(sensors[1].model_id(), None);

    Ok(())
}
