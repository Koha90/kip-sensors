//! Domain model for industrial sensors.

use std::fmt;
use std::str::FromStr;

/// Represents the current operational state of a sensor.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SensorStatus {
    Active,
    Maintenance,
    Failed,
    Offline,
}

/// Errors that may occur while creating or restoring a sensor.
#[derive(Debug, PartialEq)]
pub enum SensorError {
    EmptyName,
    EmptyLocation,

    // The supplied sensor status is not recognized.
    InvalidStatus(String),
}

/// Represents an industrial sensor.
#[derive(Debug, PartialEq)]
pub struct Sensor {
    id: u32,
    name: String,
    location: String,
    status: SensorStatus,

    model_id: Option<u32>,
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
            model_id: None,
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

    /// Restores a sensor from previously persisted data.
    ///
    /// Applies the same name and location validation as `new()`,
    /// while preserving the stored operational status.
    ///
    /// # Errors
    ///
    /// Returns an error if the name or location is empty.
    pub fn restore(
        id: u32,
        name: String,
        location: String,
        status: SensorStatus,
        model_id: Option<u32>,
    ) -> Result<Self, SensorError> {
        let mut sensor = Self::new(id, name, location)?;

        // The regular constructor sets Active.
        // Restoring must preserve the persisted status instead.
        sensor.status = status;

        sensor.model_id = model_id;

        Ok(sensor)
    }

    /// Returns the identifier of the assigned sensor model.
    ///
    /// Returns `None` when no model has been assigned.
    pub fn model_id(&self) -> Option<u32> {
        self.model_id
    }

    /// Assigns a models identifier to this sensor.
    ///
    /// Model existence must be verified by the storage layer.
    pub fn assign_model(&mut self, model_id: u32) {
        self.model_id = Some(model_id);
    }

    /// Removes the assigned model identifier.
    pub fn clear_model(&mut self) {
        self.model_id = None;
    }
}

impl fmt::Display for SensorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorError::EmptyName => write!(f, "sensor name is empty"),
            SensorError::EmptyLocation => write!(f, "sensor location is empty"),
            SensorError::InvalidStatus(status) => {
                write!(
                    f,
                    "invalid sensor status: {}",
                    status
                )
            }
        }
    }
}

impl FromStr for SensorStatus {
    type Err = SensorError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Active" => Ok(Self::Active),
            "Maintenance" => Ok(Self::Maintenance),
            "Failed" => Ok(Self::Failed),
            "Offline" => Ok(Self::Offline),

            other => Err(SensorError::InvalidStatus(
                other.to_string(),
            )),
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

    #[test]
    fn sensor_status_round_trip() {
        let statuses = [
            SensorStatus::Active,
            SensorStatus::Maintenance,
            SensorStatus::Failed,
            SensorStatus::Offline,
        ];

        for status in statuses {
            let text = status.to_string();

            let restored = text.parse::<SensorStatus>();

            assert_eq!(restored, Ok(status));
        }
    }

    #[test]
    fn invalid_sensor_status_returns_error() {
        let result = "Archive".parse::<SensorStatus>();

        assert_eq!(
            result,
            Err(SensorError::InvalidStatus(
                "Archive".to_string(),
            ))
        );
    }

    #[test]
    fn restore_preserves_sensor_status() {
        let sensor = Sensor::restore(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
            SensorStatus::Maintenance,
            None,
        )
        .expect("sensor must be created");

        assert_eq!(sensor.id(), 1);
        assert_eq!(
            sensor.status(),
            SensorStatus::Maintenance
        );
    }

    #[test]
    fn sensor_model_can_be_assigned() -> Result<(), SensorError> {
        let mut sensor = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        )?;

        assert_eq!(sensor.model_id(), None);

        sensor.assign_model(7);

        assert_eq!(sensor.model_id(), Some(7));

        Ok(())
    }

    #[test]
    fn sensor_model_can_be_cleared() -> Result<(), SensorError> {
        let mut sensor = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        )?;

        assert_eq!(sensor.model_id(), None);

        sensor.assign_model(7);

        assert_eq!(sensor.model_id(), Some(7));

        sensor.clear_model();

        assert_eq!(sensor.model_id(), None);

        Ok(())
    }
}
