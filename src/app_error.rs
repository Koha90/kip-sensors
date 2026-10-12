//! Application-level errors for the sensor catalog.

use std::fmt::{self};

use crate::cli::CliError;
use crate::sensor::SensorError;
use crate::storage::StorageError;

/// Errors that can occur while executing an application command.
#[derive(Debug)]
pub enum AppError {
    /// Invalid command-line input.
    Cli(CliError),

    /// Sensor domain validation failed.
    Sensor(SensorError),

    /// Persistent storage operation failed.
    Storage(StorageError),

    /// The requested sensor does not exist.
    NotFound(u32),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cli(err) => write!(f, "команда: {err}"),
            Self::Sensor(err) => write!(f, "датчик: {err}"),
            Self::Storage(err) => write!(f, "база данных: {err}"),
            Self::NotFound(id) => {
                write!(f, "датчик с ID {id} не найден")
            }
        }
    }
}

impl From<CliError> for AppError {
    fn from(err: CliError) -> Self {
        Self::Cli(err)
    }
}

impl From<SensorError> for AppError {
    fn from(err: SensorError) -> Self {
        Self::Sensor(err)
    }
}

impl From<StorageError> for AppError {
    fn from(err: StorageError) -> Self {
        Self::Storage(err)
    }
}
