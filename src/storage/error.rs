//! Error types for SQLite storage operations.

use std::fmt;

use crate::sensor::SensorError;
use crate::sensor_model::SensorModelError;

/// Errors that may occur while accessing stored sensors.
#[derive(Debug)]
pub enum StorageError {
    /// An SQLite operation failed.
    Database(rusqlite::Error),

    /// Stored data violates the sensor domain rules.
    InvalidSensor(SensorError),

    /// Database schema migration failed.
    Migration(rusqlite_migration::Error),

    /// Stored model data violates domain rules.
    InvalidModel(SensorModelError),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(err) => {
                write!(f, "database error: {err}")
            }

            Self::InvalidSensor(err) => {
                write!(f, "invalid stored sensor: {err}")
            }

            Self::Migration(err) => {
                write!(
                    f,
                    "database migration error: {err}"
                )
            }

            Self::InvalidModel(err) => {
                write!(
                    f,
                    "invalid stored sensor model: {err}"
                )
            }
        }
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Database(err)
    }
}

impl From<SensorError> for StorageError {
    fn from(err: SensorError) -> Self {
        Self::InvalidSensor(err)
    }
}

impl From<rusqlite_migration::Error> for StorageError {
    fn from(err: rusqlite_migration::Error) -> Self {
        Self::Migration(err)
    }
}

impl From<SensorModelError> for StorageError {
    fn from(err: SensorModelError) -> Self {
        Self::InvalidModel(err)
    }
}

/// Result type for sensor storage operations.
pub type StorageResult<T> = Result<T, StorageError>;
