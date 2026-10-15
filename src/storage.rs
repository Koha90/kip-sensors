//! SQLite persistence for the sensor catalog.

mod error;
mod models;
mod sensors;
mod summaries;

pub use error::{StorageError, StorageResult};
pub use summaries::SensorSummary;

use std::path::Path;

use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

/// Ordered database schema migrations.
///
/// Migration positions correspond to database schema versions.
/// Existing migrations must not be reordered or modified
/// after they have been applied.
const MIGRATION_STEPS: &[M<'static>] = &[
    M::up(include_str!(
        "../migrations/001_create_sensors.sql"
    )),
    M::up(include_str!(
        "../migrations/002_create_sensor_models.sql"
    )),
];

/// Database migrator used to apply pending schema changes.
const MIGRATIONS: Migrations<'static> = Migrations::from_slice(MIGRATION_STEPS);

/// Manages the SQLite database used by the sensor catalog.
pub struct SqliteSensorStore {
    conn: Connection,
}

impl SqliteSensorStore {
    /// Opens a SQLite database and applies pending migrations.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened,
    /// migrated, or has an unsupported schema version.
    pub fn open(path: impl AsRef<Path>) -> StorageResult<Self> {
        let mut conn = Connection::open(path)?;

        MIGRATIONS.to_latest(&mut conn)?;

        conn.pragma_update(None, "foreign_keys", "ON")?;

        Ok(Self { conn })
    }
}

#[cfg(test)]
mod tests;
