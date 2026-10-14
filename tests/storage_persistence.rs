use kip_sensors::sensor::{Sensor, SensorStatus};
use kip_sensors::sensor_model::SensorModel;
use kip_sensors::storage::{SqliteSensorStore, StorageResult};

/// Verifies that sensor data survives closing and reopening
/// a file-blocked SQLite database.
#[test]
fn sensor_survives_database_reopen() -> StorageResult<()> {
    let dir = tempfile::tempdir().expect("temporary directory must be created");

    let db_path = dir.path().join("sensors.db");

    // First connection: persist a sensor.
    {
        let store = SqliteSensorStore::open(&db_path)?;

        let mut sensor = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        )
        .expect("sensor must be created");

        sensor.maintenance();

        store.insert(&sensor)?;
    }

    // Second connection: restore the persisted sensor.
    let store = SqliteSensorStore::open(&db_path)?;

    let sensor = store
        .find(1)?
        .expect("sensor must survive database reopening");

    assert_eq!(sensor.id(), 1);
    assert_eq!(sensor.name(), "PT-101");
    assert_eq!(sensor.location(), "Пастеризатор");
    assert_eq!(
        sensor.status(),
        SensorStatus::Maintenance
    );

    Ok(())
}

#[test]
fn sensor_model_link_survives_database_reopen() -> StorageResult<()> {
    let dir = tempfile::tempdir().expect("temporary directory must be created");

    let db_path = dir.path().join("sensors.db");

    {
        let store = SqliteSensorStore::open(&db_path)?;

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
    }

    let store = SqliteSensorStore::open(&db_path)?;

    let sensor = store
        .find(1)?
        .expect("sensor must survive database reopening");

    assert_eq!(sensor.id(), 1);
    assert_eq!(sensor.name(), "PT-101");
    assert_eq!(sensor.location(), "Пастеризатор");
    assert_eq!(
        sensor.status(),
        SensorStatus::Active
    );
    assert_eq!(sensor.model_id(), Some(7));

    let model = store
        .find_model(7)?
        .expect("sensor model must survive database reopening");

    assert_eq!(model.id(), 7);
    assert_eq!(model.manufacturer(), "WIKA");
    assert_eq!(model.model(), "A-10");
    assert_eq!(model.part_number(), None);

    Ok(())
}
