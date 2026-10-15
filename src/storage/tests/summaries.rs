use super::super::{SqliteSensorStore, StorageResult};

use crate::sensor::Sensor;
use crate::sensor_model::SensorModel;

#[test]
fn list_with_models_includes_sensors_without_model() -> StorageResult<()> {
    let store = SqliteSensorStore::open(":memory:")?;

    let mut sensors = [
        Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор, линия 1".to_string(),
        )?,
        Sensor::new(
            2,
            "TT-303".to_string(),
            "Танк хранения".to_string(),
        )?,
    ];

    let model = SensorModel::new(
        7,
        "WIKA".to_string(),
        "A-10".to_string(),
        Some("A-10-123".to_string()),
    )?;

    sensors[0].assign_model(model.id());

    store.insert_model(&model)?;

    for sensor in &sensors {
        store.insert(sensor)?;
    }

    let list_sensors = store.list_with_models()?;

    assert_eq!(list_sensors.len(), 2);

    assert_eq!(list_sensors[0].name, "PT-101");
    assert_eq!(
        list_sensors[0]
            .manufacturer
            .as_deref(),
        Some("WIKA")
    );
    assert_eq!(
        list_sensors[0]
            .model
            .as_deref(),
        Some("A-10")
    );
    assert_eq!(list_sensors[1].name, "TT-303");
    assert_eq!(list_sensors[1].manufacturer, None);
    assert_eq!(list_sensors[1].model, None);

    Ok(())
}
