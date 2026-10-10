use std::fmt;

use crate::sensor::Sensor;

#[derive(Debug, PartialEq)]
pub enum CatalogError {
    DuplicateId(u32),
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CatalogError::DuplicateId(id) => {
                write!(
                    f,
                    "sensor with id {} already exists",
                    id
                )
            }
        }
    }
}

/// Stores and manages sensors.
#[derive(Default)]
pub struct SensorCatalog {
    sensors: Vec<Sensor>,
}

impl SensorCatalog {
    /// Creates an empty sensor catalog.
    pub fn new() -> Self {
        Self {
            sensors: Vec::new(),
        }
    }

    /// Adds a sensor to the catalog.
    pub fn add(&mut self, sensor: Sensor) -> Result<(), CatalogError> {
        let id = sensor.id();

        let exists = self
            .sensors
            .iter()
            .any(|item| item.id() == id);

        if exists {
            return Err(CatalogError::DuplicateId(id));
        }

        self.sensors.push(sensor);

        Ok(())
    }

    /// Returns an immutable view of all sensors.
    pub fn sensors(&self) -> &[Sensor] {
        &self.sensors
    }

    /// Finds a sensor by its ID.
    pub fn find(&self, id: u32) -> Option<&Sensor> {
        self.sensors
            .iter()
            .find(|sensor| sensor.id() == id)
    }

    /// Finds a mutable sensor by its ID.
    pub fn find_mut(&mut self, id: u32) -> Option<&mut Sensor> {
        self.sensors
            .iter_mut()
            .find(|sensor| sensor.id() == id)
    }

    /// Removes a sensor by its ID.
    pub fn remove(&mut self, id: u32) -> Option<Sensor> {
        let position = self
            .sensors
            .iter()
            .position(|sensor| sensor.id() == id);

        match position {
            Some(index) => Some(self.sensors.remove(index)),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sensor_success() {
        let mut catalog = SensorCatalog::new();

        let sensor = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        )
        .expect("sensor must be created");

        let result = catalog.add(sensor);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn add_duplicated_sensor() {
        let mut catalog = SensorCatalog::new();

        let sensor1 = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        )
        .expect("sensor must be created");

        let sensor2 = Sensor::new(
            1,
            "PT-101".to_string(),
            "Танк хранения".to_string(),
        )
        .expect("sensor must be created");

        catalog
            .add(sensor1)
            .expect("first sensor must be added");

        let result = catalog.add(sensor2);

        assert_eq!(
            result,
            Err(CatalogError::DuplicateId(1))
        );
    }

    #[test]
    fn find_sensor_by_id() {
        let mut catalog = SensorCatalog::new();

        let sensor = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        )
        .expect("sensor must be created");

        catalog
            .add(sensor)
            .expect("sensor must be added");

        let sensor = catalog
            .find(1)
            .expect("sensor must be found");

        assert_eq!(sensor.id(), 1);
        assert_eq!(sensor.name(), "PT-101");
    }

    #[test]
    fn find_missing_sensor_returns_none() {
        let catalog = SensorCatalog::new();

        assert_eq!(catalog.find(42), None);
    }

    #[test]
    fn remove_sensor_success() {
        let mut catalog = SensorCatalog::new();

        let sensor = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        )
        .expect("sensor must be created");

        catalog
            .add(sensor)
            .expect("sensor must be added");

        let removed = catalog
            .remove(1)
            .expect("sensor must be removed");

        assert_eq!(removed.id(), 1);
        assert_eq!(catalog.find(1), None);
    }

    #[test]
    fn remove_missing_sensor_returns_none() {
        let mut catalog = SensorCatalog::new();

        assert_eq!(catalog.remove(42), None)
    }

    #[test]
    fn default_catalog_is_empty() {
        let catalog = SensorCatalog::default();

        assert!(catalog.sensors().is_empty());
    }
}
