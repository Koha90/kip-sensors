use super::super::{SqliteSensorStore, StorageError, StorageResult};

use crate::sensor_model::{SensorModel, SensorModelError};

use rusqlite::params;

#[test]
fn insert_sensor_model_success() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let sensor_model = SensorModel::new(
        1,
        "WIKA".to_string(),
        "A-10".to_string(),
        Some("A-10-123".to_string()),
    )
    .expect("sensor model must be created");

    store.insert_model(&sensor_model)?;

    let (manufacturer, model, part_number): (String, String, Option<String>) =
        store.conn.query_row(
            "SELECT manufacturer, model, part_number FROM sensor_models WHERE id = ?1",
            [1],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                ))
            },
        )?;

    assert_eq!(manufacturer, "WIKA");
    assert_eq!(model, "A-10");
    assert_eq!(
        part_number,
        Some("A-10-123".to_string())
    );

    Ok(())
}

#[test]
fn find_existing_sensor_model() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let sensor_model = SensorModel::new(
        1,
        "WIKA".to_string(),
        "A-10".to_string(),
        Some("A-10-123".to_string()),
    )
    .expect("sensor model must be created");

    store.insert_model(&sensor_model)?;

    let restored = store
        .find_model(1)?
        .expect("sensor model must be found");

    assert_eq!(restored.id(), 1);
    assert_eq!(restored.manufacturer(), "WIKA");
    assert_eq!(restored.model(), "A-10");
    assert_eq!(
        restored.part_number(),
        Some("A-10-123"),
    );

    Ok(())
}

#[test]
fn find_invalid_sensor_model_returns_error() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    // Bypass the domain constructor to simulate invalid stored data.
    store.conn.execute(
        "INSERT INTO sensor_models
             (id, manufacturer, model, part_number)
             VALUES (?1, ?2, ?3, NULL)",
        params![1, "", "A-10"],
    )?;

    // Restoring the record must fail domain validation.
    let result = store.find_model(1);

    assert!(matches!(
        result,
        Err(StorageError::InvalidModel(
            SensorModelError::EmptyManufacturer
        ))
    ));

    Ok(())
}

#[test]
fn insert_sensor_model_without_part_number() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let sensor_model = SensorModel::new(
        1,
        "WIKA".to_string(),
        "A-10".to_string(),
        None,
    )
    .expect("sensor model must be created");

    store.insert_model(&sensor_model)?;

    let part_number: Option<String> = store.conn.query_row(
        "SELECT part_number FROM sensor_models WHERE id = ?1",
        [1],
        |row| row.get(0),
    )?;

    assert!(part_number.is_none());

    Ok(())
}

#[test]
fn insert_duplicate_id_sensor_model_returns_error() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let sensor_model = SensorModel::new(
        1,
        "WIKA".to_string(),
        "A-10".to_string(),
        Some("A-10-123".to_string()),
    )
    .expect("sensor model must be created");

    store.insert_model(&sensor_model)?;

    let result = store.insert_model(&sensor_model);

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
fn find_model_without_part_number() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let model = SensorModel::new(
        1,
        "WIKA".to_string(),
        "A-10".to_string(),
        None,
    )
    .expect("sensor model must be created");

    store.insert_model(&model)?;

    let restored = store
        .find_model(1)?
        .expect("sensor model must be found");

    assert_eq!(restored.id(), 1);
    assert_eq!(restored.manufacturer(), "WIKA");
    assert_eq!(restored.model(), "A-10");
    assert_eq!(restored.part_number(), None);

    Ok(())
}

#[test]
fn find_missing_sensor_model_returns_none() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let sensor_model = SensorModel::new(
        1,
        "WIKA".to_string(),
        "A-10".to_string(),
        Some("A-10-123".to_string()),
    )
    .expect("sensor model must be created");

    store.insert_model(&sensor_model)?;

    let result = store.find_model(42)?;

    assert!(result.is_none());

    Ok(())
}
