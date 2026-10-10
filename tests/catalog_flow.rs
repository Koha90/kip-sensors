use kip_sensors::catalog::SensorCatalog;
use kip_sensors::sensor::Sensor;

#[test]
fn sensor_catalog_flow() {
    let mut catalog = SensorCatalog::new();

    let sensor = Sensor::new(
        1,
        "PT-101".to_string(),
        "Пастеризатор, линия 1".to_string(),
    )
    .expect("sensor must be created");

    catalog
        .add(sensor)
        .expect("sensor must be added");

    let sensor = catalog
        .find_mut(1)
        .expect("sensor must be found");

    sensor
        .rename("PT-101A")
        .expect("sensor must be renamed");

    sensor
        .relocate("Пастеризатор, линия 2")
        .expect("sensor must be relocated");

    let sensor = catalog
        .find(1)
        .expect("sensor must be exist");

    assert_eq!(sensor.name(), "PT-101A");
    assert_eq!(
        sensor.location(),
        "Пастеризатор, линия 2"
    );
}
