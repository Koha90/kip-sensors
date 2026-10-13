//! SQLite persistence for the sensor catalog.

use std::fmt;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Params, params};

use crate::sensor::{Sensor, SensorError, SensorStatus};
use crate::sensor_model::{SensorModel, SensorModelError};

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

    /// Inserts a sensor into the SQLite database.
    ///
    /// # Errors
    ///
    /// Returns an error if the insert fails, including when
    /// a sensor with the same ID already exists.
    pub fn insert(&self, sensor: &Sensor) -> StorageResult<()> {
        self.conn.execute(
            "INSERT INTO sensors (id, name, location, status)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                sensor.id(),
                sensor.name(),
                sensor.location(),
                sensor.status().to_string(),
            ],
        )?;

        Ok(())
    }

    /// Finds a stored sensor by its ID.
    ///
    /// Returns `Ok(None)` if no sensor has the requested ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails or stored data
    /// cannot be restored into a valid Sensor.
    pub fn find(&self, id: u32) -> StorageResult<Option<Sensor>> {
        let record: Option<(u32, String, String, String)> = self
            .conn
            .query_row(
                "SELECT id, name, location, status
                 FROM sensors
                 WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                    ))
                },
            )
            .optional()?;

        let Some((id, name, location, status)) = record else {
            return Ok(None);
        };

        let status = status.parse::<SensorStatus>()?;

        let sensor = Sensor::restore(id, name, location, status)?;

        Ok(Some(sensor))
    }

    /// Returns all stored sensors ordered by ID.
    ///
    /// Returns an empty vector when the database contains no sensors.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails or a stored
    /// sensor cannot be restored.
    pub fn list(&self) -> StorageResult<Vec<Sensor>> {
        self.query_sensor(
            "SELECT id, name, location, status
             FROM sensors
             ORDER BY id",
            [],
        )
    }

    /// Searches sensors by name or location.
    ///
    /// Performs a case-sensitive literal substring search.
    /// Leading and trailing whitespace in the query is ignored.
    /// Returns an empty vector for an empty query.
    ///
    /// Results are ordered by sensor ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails or a stored
    /// sensor cannot be restored.
    pub fn search(&self, query: &str) -> StorageResult<Vec<Sensor>> {
        let query = query.trim();

        if query.is_empty() {
            return Ok(Vec::new());
        }

        self.query_sensor(
            "SELECT id, name, location, status
             FROM sensors
             WHERE instr(name, ?1) > 0
                OR instr(location, ?1) > 0
             ORDER BY id",
            params![query],
        )
    }

    /// Deletes a stored sensor by its ID.
    ///
    /// Returns `true` if a sensor was deleted, or `false`
    /// if no sensor with the given ID exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails..
    pub fn remove(&self, id: u32) -> StorageResult<bool> {
        let affected = self.conn.execute(
            "DELETE FROM sensors WHERE id = ?1",
            [id],
        )?;

        Ok(affected > 0)
    }

    /// Executes a sensor query and restores its rows as domain objects.
    ///
    /// All returned records pass through Sensor::restore(),
    /// so invalid persisted data cannot silently enter the domain.
    fn query_sensor(&self, sql: &str, parameters: impl Params) -> StorageResult<Vec<Sensor>> {
        let mut stmt = self.conn.prepare(sql)?;

        let rows = stmt.query_map(parameters, |row| {
            Ok((
                row.get::<_, u32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        let mut sensors = Vec::new();

        for row in rows {
            let (id, name, location, raw_status) = row?;

            // Convert SQL text into a validated domain status.
            let status = raw_status.parse::<SensorStatus>()?;

            let sensor = Sensor::restore(id, name, location, status)?;

            sensors.push(sensor);
        }

        Ok(sensors)
    }

    /// Inserts a sensor model into the database.
    ///
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub fn insert_model(&self, model: &SensorModel) -> StorageResult<()> {
        self.conn.execute(
            "INSERT INTO sensor_models (id, manufacturer, model, part_number)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                model.id(),
                model.manufacturer(),
                model.model(),
                model.part_number(),
            ],
        )?;

        Ok(())
    }

    /// Finds a stored sensor model by its ID.
    ///
    /// Returns `None` if the model does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails
    /// or stored model data violates domain rules.
    pub fn find_model(&self, id: u32) -> StorageResult<Option<SensorModel>> {
        let record: Option<(u32, String, String, Option<String>)> = self
            .conn
            .query_row(
                "SELECT id, manufacturer, model, part_number
                 FROM sensor_models
                 WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                    ))
                },
            )
            .optional()?;

        let Some((id, manufacturer, model, part_number)) = record else {
            return Ok(None);
        };

        // Restore the validated domain model.
        let sensor_model = SensorModel::new(
            id,
            manufacturer,
            model,
            part_number,
        )?;

        Ok(Some(sensor_model))
    }
}

/// Errors that may occur while accessing stored sensors.
#[derive(Debug)]
pub enum StorageError {
    /// An SQLite operation failed.
    Database(rusqlite::Error),

    /// Stored data violates the sensor domain rules.
    InvalidSensor(SensorError),

    /// Database schema migration failed.
    Migration(rusqlite_migration::Error),

    /// Store model data violates domain rules.
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
                    "invalid store sensor model: {err}"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_creates_sensors_table() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let count: i64 = store.conn.query_row(
            "SELECT COUNT(*)
             FROM sqlite_master
             WHERE type = 'table'
               AND name = 'sensors'",
            [],
            |row| row.get(0),
        )?;

        assert_eq!(count, 1);

        Ok(())
    }

    #[test]
    fn insert_sensor_success() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let mut sensor = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        )
        .expect("sensor must be created");

        sensor.maintenance();

        store.insert(&sensor)?;

        let (name, status): (String, String) = store.conn.query_row(
            "SELECT name, status FROM sensors WHERE id = ?1",
            [1],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        assert_eq!(name, "PT-101");
        assert_eq!(status, "Maintenance");

        Ok(())
    }

    #[test]
    fn insert_duplicate_id_returns_error() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let sensor = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        )
        .expect("sensor must be created");

        // The first insertion must succeed.
        store.insert(&sensor)?;

        // The second insertion must violate the primary key.
        let result = store.insert(&sensor);

        assert!(matches!(
                result,
                Err(StorageError::Database(
                        rusqlite::Error::SqliteFailure(err, _)
                ))
                    if err.extended_code
                        == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY
        ));

        Ok(())
    }

    #[test]
    fn find_existing_sensor() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let mut sensor = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        )
        .expect("sensor must be created");

        sensor.maintenance();

        store.insert(&sensor)?;

        let restored = store
            .find(1)?
            .expect("sensor must be found");

        assert_eq!(restored.id(), 1);
        assert_eq!(restored.name(), "PT-101");
        assert_eq!(restored.location(), "Пастеризатор");
        assert_eq!(
            restored.status(),
            SensorStatus::Maintenance
        );

        Ok(())
    }

    #[test]
    fn find_missing_sensor_returns_none() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let result = store.find(42)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn list_empty_database_returns_empty_vec() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let sensors = store.list()?;

        assert!(sensors.is_empty());

        Ok(())
    }

    #[test]
    fn list_returns_sensors_ordered_by_id() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let mut first = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        )?;

        let second = Sensor::new(
            2,
            "PT-202".to_string(),
            "Танк хранения".to_string(),
        )?;

        first.maintenance();

        // Insert in reverse order to verify SQL sorting.
        store.insert(&second)?;
        store.insert(&first)?;

        let sensors = store.list()?;

        assert_eq!(sensors.len(), 2);

        assert_eq!(sensors[0].id(), 1);
        assert_eq!(sensors[0].name(), "PT-101");
        assert_eq!(
            sensors[0].status(),
            SensorStatus::Maintenance
        );

        assert_eq!(sensors[1].id(), 2);
        assert_eq!(sensors[1].name(), "PT-202");
        assert_eq!(
            sensors[1].status(),
            SensorStatus::Active
        );

        Ok(())
    }

    #[test]
    fn delete_sensor_by_its_id_succeed() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let sensor = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        )?;

        // Arrange: persist a sensor before attempting deletion.
        store.insert(&sensor)?;

        // Act: delete the stored sensor.
        let removed = store.remove(1)?;

        // Assert: deletion succeeded and the record is gone.
        assert!(removed);
        assert!(store.find(1)?.is_none());

        Ok(())
    }

    #[test]
    fn remove_missing_sensor_returns_false() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let removed = store.remove(42)?;

        assert!(!removed);

        Ok(())
    }

    #[test]
    fn search_matches_name_and_location() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let sensors = [
            Sensor::new(
                1,
                "PT-101".to_string(),
                "Пастеризатор, линия 1".to_string(),
            )?,
            Sensor::new(
                2,
                "PT-202".to_string(),
                "Танк хранения".to_string(),
            )?,
            Sensor::new(
                3,
                "TT-303".to_string(),
                "Пастеризатор, линия 2".to_string(),
            )?,
        ];

        // Arrange: persist all test sensors.
        for sensor in &sensors {
            store.insert(sensor)?;
        }

        // Act: search by installation location.
        let by_location = store.search("Пастеризатор")?;

        assert_eq!(by_location.len(), 2);
        assert_eq!(by_location[0].id(), 1);
        assert_eq!(by_location[1].id(), 3);

        // Act: search by sensor name.
        let by_name = store.search("PT")?;

        assert_eq!(by_name.len(), 2);
        assert_eq!(by_name[0].id(), 1);
        assert_eq!(by_name[1].id(), 2);

        Ok(())
    }

    #[test]
    fn search_returns_empty_for_unknown_query() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let sensor = Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор".to_string(),
        )?;

        store.insert(&sensor)?;

        assert!(
            store
                .search("Несуществующий")?
                .is_empty()
        );
        assert!(store.search("")?.is_empty());

        // A percent sign is searched literally, not as a LIKE wildcard.
        assert!(store.search("%")?.is_empty());

        Ok(())
    }

    #[test]
    fn open_initializes_schema_version() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let version: i64 = store
            .conn
            .pragma_query_value(None, "user_version", |row| {
                row.get(0)
            })?;

        assert_eq!(version, 2);

        Ok(())
    }

    #[test]
    fn unfinished_transaction_rolls_back() -> StorageResult<()> {
        let mut store = SqliteSensorStore::open(":memory:")?;

        {
            let tx = store.conn.transaction()?;

            tx.execute(
                "INSERT INTO sensors (id, name, location, status)
                 VALUES (?1, ?2, ?3, ?4)",
                params![1, "PT-101", "Пастеризатор", "Active"],
            )?;

            // No commit: the transaction is dropped here.
        }

        // The insered sensor must not exist after rollback.
        assert!(store.find(1)?.is_none());

        Ok(())
    }

    #[test]
    fn open_rejects_unsupported_schema_version() -> StorageResult<()> {
        // Create an isolated temporary directory.
        let dir = tempfile::tempdir().expect("temporary directory must be created");

        // Build the database file path.
        let db_path = dir
            .path()
            .join("sensors_test.db");

        // Create a database with an unsuppoted schema version.
        let conn = Connection::open(&db_path)?;

        conn.pragma_update(None, "user_version", 99_i64)?;

        // Close the connection before reopening the database.
        drop(conn);

        // Attempt to open the database through our storage API.
        let result = SqliteSensorStore::open(&db_path);

        // Verify the exact error variant and its version.
        assert!(matches!(
            result,
            Err(StorageError::Migration(
                rusqlite_migration::Error::MigrationDefinition(
                    rusqlite_migration::MigrationDefinitionError::DatabaseTooFarAhead
                )
            ))
        ));

        Ok(())
    }

    #[test]
    fn opening_database_after_tempdir_drop_fails() {
        let dir = tempfile::tempdir().unwrap();

        let db_path = dir.path().join("sensors.db");

        drop(dir);

        let result = rusqlite::Connection::open(&db_path);

        assert!(result.is_err());
    }

    #[test]
    fn migrations_are_valid() {
        MIGRATIONS
            .validate()
            .expect("all migrations must be valid");
    }

    #[test]
    fn insert_sensor_model_success() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let sensor_model = SensorModel::new(
            1,
            "WIKA".to_string(),
            "A-10".to_string(),
            Some("A-10-123".to_string()),
        )
        .expect("sensor model must be created");

        store.insert_model(&sensor_model)?;

        let (manufacturer, model, part_number): (String, String, Option<String>) =
            store.conn.query_row(
                "SELECT manufacturer, model, part_number FROM sensor_models WHERE id = ?1",
                [1],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                    ))
                },
            )?;

        assert_eq!(manufacturer, "WIKA");
        assert_eq!(model, "A-10");
        assert_eq!(
            part_number,
            Some("A-10-123".to_string())
        );

        Ok(())
    }

    #[test]
    fn insert_sensor_model_without_part_number() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let sensor_model = SensorModel::new(
            1,
            "WIKA".to_string(),
            "A-10".to_string(),
            None,
        )
        .expect("sensor model must be created");

        store.insert_model(&sensor_model)?;

        let part_number: Option<String> = store.conn.query_row(
            "SELECT part_number FROM sensor_models WHERE id = ?1",
            [1],
            |row| row.get(0),
        )?;

        assert!(part_number.is_none());

        Ok(())
    }

    #[test]
    fn insert_duplicate_id_sensor_model_returns_error() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let sensor_model = SensorModel::new(
            1,
            "WIKA".to_string(),
            "A-10".to_string(),
            Some("A-10-123".to_string()),
        )
        .expect("sensor model must be created");

        store.insert_model(&sensor_model)?;

        let result = store.insert_model(&sensor_model);

        assert!(matches!(
                result,
                Err(StorageError::Database(
                        rusqlite::Error::SqliteFailure(err, _)
                ))
                    if err.extended_code
                        == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY
        ));

        Ok(())
    }

    #[test]
    fn find_model_without_part_number() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let model = SensorModel::new(
            1,
            "WIKA".to_string(),
            "A-10".to_string(),
            None,
        )
        .expect("sensor model must be created");

        store.insert_model(&model)?;

        let restored = store
            .find_model(1)?
            .expect("sensor model must be found");

        assert_eq!(restored.id(), 1);
        assert_eq!(restored.manufacturer(), "WIKA");
        assert_eq!(restored.model(), "A-10");
        assert_eq!(restored.part_number(), None);

        Ok(())
    }

    #[test]
    fn find_existing_sensor_model() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let sensor_model = SensorModel::new(
            1,
            "WIKA".to_string(),
            "A-10".to_string(),
            Some("A-10-123".to_string()),
        )
        .expect("sensor model must be created");

        store.insert_model(&sensor_model)?;

        let restored = store
            .find_model(1)?
            .expect("sensor model must be found");

        assert_eq!(restored.id(), 1);
        assert_eq!(restored.manufacturer(), "WIKA");
        assert_eq!(restored.model(), "A-10");
        assert_eq!(
            restored.part_number(),
            Some("A-10-123"),
        );

        Ok(())
    }

    #[test]
    fn find_missing_sensor_model_returns_none() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        let sensor_model = SensorModel::new(
            1,
            "WIKA".to_string(),
            "A-10".to_string(),
            Some("A-10-123".to_string()),
        )
        .expect("sensor model must be created");

        store.insert_model(&sensor_model)?;

        let result = store.find_model(42)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn find_invalid_sensor_model_returns_error() -> StorageResult<()> {
        let store = SqliteSensorStore::open(":memory:")?;

        // Bypass the domain constructor to simulate invalid stored data.
        store.conn.execute(
            "INSERT INTO sensor_models
             (id, manufacturer, model, part_number)
             VALUES (?1, ?2, ?3, NULL)",
            params![1, "", "A-10"],
        )?;

        // Restoring the record must fail domain validation.
        let result = store.find_model(1);

        assert!(matches!(
            result,
            Err(StorageError::InvalidModel(
                SensorModelError::EmptyManufacturer
            ))
        ));

        Ok(())
    }
}
