use kip_sensors::sensor::{Sensor, SensorStatus};
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
