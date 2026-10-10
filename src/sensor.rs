//! Domain model for industrial sensors.

use std::fmt;

/// Represents the current operational state of a sensor.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SensorStatus {
    Active,
    Maintenance,
    Failed,
    Offline,
}

/// Errors that may occur while creating a sensor.
#[derive(Debug, PartialEq)]
pub enum SensorError {
    EmptyName,
    EmptyLocation,
}

/// Represents an industrial sensor.
#[derive(Debug, PartialEq)]
pub struct Sensor {
    id: u32,
    name: String,
    location: String,
    status: SensorStatus,
}

impl fmt::Display for SensorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorStatus::Active => write!(f, "Active"),
            SensorStatus::Maintenance => write!(f, "Maintenance"),
            SensorStatus::Failed => write!(f, "Failed"),
            SensorStatus::Offline => write!(f, "Offline"),
        }
    }
}

impl Sensor {
    /// Creates a new sensor.
    ///
    /// Returns an error if the name or location is empty.
    pub fn new(id: u32, name: String, location: String) -> Result<Sensor, SensorError> {
        if name.trim().is_empty() {
            return Err(SensorError::EmptyName);
        }

        if location.trim().is_empty() {
            return Err(SensorError::EmptyLocation);
        }

        Ok(Sensor {
            id,
            name,
            location,
            status: SensorStatus::Active,
        })
    }

    /// Marks the sensor as failed.
    pub fn fail(&mut self) {
        self.status = SensorStatus::Failed;
    }

    pub fn maintenance(&mut self) {
        self.status = SensorStatus::Maintenance;
    }

    pub fn offline(&mut self) {
        self.status = SensorStatus::Offline;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Changes the sensor name.
    ///
    /// Returns an error if the new name is empty.
    pub fn rename(&mut self, name: &str) -> Result<(), SensorError> {
        if name.trim().is_empty() {
            return Err(SensorError::EmptyName);
        }

        self.name = name.to_string();

        Ok(())
    }

    pub fn location(&self) -> &str {
        &self.location
    }

    /// Changes the sensor location.
    ///
    /// Returns an error if the new location is empty.
    pub fn relocate(&mut self, location: &str) -> Result<(), SensorError> {
        if location.trim().is_empty() {
            return Err(SensorError::EmptyLocation);
        }

        self.location = location.to_string();

        Ok(())
    }

    pub fn status(&self) -> SensorStatus {
        self.status
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

impl fmt::Display for SensorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorError::EmptyName => write!(f, "sensor name is empty"),
            SensorError::EmptyLocation => write!(f, "sensor location is empty"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sensor_success() {
        let result = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        );

        let sensor = result.expect("sensor must be created");

        assert_eq!(sensor.id(), 1);
        assert_eq!(sensor.name(), "PT-101");
        assert_eq!(sensor.location(), "Пастеризатор");
    }

    #[test]
    fn new_sensor_empty_name() {
        let result = Sensor::new(
            1,
            "   ".to_string(),
            "Пастеризатор".to_string(),
        );

        assert_eq!(result, Err(SensorError::EmptyName));
    }

    #[test]
    fn new_sensor_empty_location() {
        let result = Sensor::new(
            1,
            "PT-101".to_string(),
            "  ".to_string(),
        );

        assert_eq!(
            result,
            Err(SensorError::EmptyLocation)
        );
    }
}
