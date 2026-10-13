//! Domain model for sensor types and their shared specifications.

use std::fmt;

/// Errors that can occur while creating a sensor model.
#[derive(Debug, PartialEq, Eq)]
pub enum SensorModelError {
    /// Manufacturer name must not be empty.
    EmptyManufacturer,

    /// Model name must not be empty.
    EmptyModel,
}

impl fmt::Display for SensorModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyManufacturer => {
                write!(f, "manufacturer must not be empty")
            }
            Self::EmptyModel => {
                write!(f, "model name must not be empty")
            }
        }
    }
}

/// Represents a sensor model shared by installed sensors.
#[derive(Debug, PartialEq, Eq)]
pub struct SensorModel {
    id: u32,
    manufacturer: String,
    model: String,
    part_number: Option<String>,
}

impl SensorModel {
    /// Creates a validated sensor model.
    ///
    /// # Errors
    ///
    /// Returns an error if the manufacturer or model is blank.
    pub fn new(
        id: u32,
        manufacturer: String,
        model: String,
        part_number: Option<String>,
    ) -> Result<Self, SensorModelError> {
        if manufacturer.trim().is_empty() {
            return Err(SensorModelError::EmptyManufacturer);
        }

        if model.trim().is_empty() {
            return Err(SensorModelError::EmptyModel);
        }

        Ok(Self {
            id,
            manufacturer,
            model,
            part_number,
        })
    }

    /// Returns the model identifier.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the manufacturer name.
    pub fn manufacturer(&self) -> &str {
        &self.manufacturer
    }

    /// Returns the model name.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Returns the optional manufacturer part number.
    pub fn part_number(&self) -> Option<&str> {
        self.part_number.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sensor_model_success() {
        let model = SensorModel::new(
            1,
            "WIKA".to_string(),
            "A-10".to_string(),
            Some("A-10-123".to_string()),
        )
        .expect("sensor model must be created");

        assert_eq!(model.id(), 1);
        assert_eq!(model.manufacturer(), "WIKA");
        assert_eq!(model.model(), "A-10");
        assert_eq!(
            model.part_number(),
            Some("A-10-123")
        );
    }

    #[test]
    fn new_sensor_model_without_part_number() {
        let model = SensorModel::new(
            1,
            "WIKA".to_string(),
            "A-10".to_string(),
            None,
        )
        .expect("sensor model must be created");

        assert_eq!(model.id(), 1);
        assert_eq!(model.manufacturer(), "WIKA");
        assert_eq!(model.model(), "A-10");
        assert_eq!(model.part_number(), None);
    }

    #[test]
    fn new_sensor_model_with_empty_manufacturer_returns_error() {
        let result = SensorModel::new(
            1,
            "   ".to_string(),
            "A-10".to_string(),
            Some("A-10-123".to_string()),
        );

        assert!(matches!(
            result,
            Err(SensorModelError::EmptyManufacturer)
        ));
    }

    #[test]
    fn new_sensor_model_with_empty_model_returns_error() {
        let result = SensorModel::new(
            1,
            "WIKA".to_string(),
            "    ".to_string(),
            Some("A-10-123".to_string()),
        );

        assert!(matches!(
            result,
            Err(SensorModelError::EmptyModel)
        ));
    }
}
