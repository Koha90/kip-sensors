//! SQLite persistence operations for installed sensors.

use rusqlite::{OptionalExtension, Params, params};

use crate::sensor::{Sensor, SensorStatus};

use super::{SqliteSensorStore, StorageResult};

impl SqliteSensorStore {
    /// Inserts a sensor into the SQLite database.
    ///
    /// # Errors
    ///
    /// Returns an error if the insert fails, including
    /// duplicate IDs or invalid model references.
    pub fn insert(&self, sensor: &Sensor) -> StorageResult<()> {
        self.conn.execute(
            "INSERT INTO sensors (id, name, location, status, model_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                sensor.id(),
                sensor.name(),
                sensor.location(),
                sensor.status().to_string(),
                sensor.model_id(),
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
        let record: Option<(
            u32,
            String,
            String,
            String,
            Option<u32>,
        )> = self
            .conn
            .query_row(
                "SELECT id, name, location, status, model_id
                 FROM sensors
                 WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .optional()?;

        let Some((id, name, location, status, model_id)) = record else {
            return Ok(None);
        };

        let status = status.parse::<SensorStatus>()?;

        let sensor = Sensor::restore(
            id, name, location, status, model_id,
        )?;

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
            "SELECT id, name, location, status, model_id
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
            "SELECT id, name, location, status, model_id
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
    /// Returns an error if the database operation fails.
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
                row.get::<_, Option<u32>>(4)?,
            ))
        })?;

        let mut sensors = Vec::new();

        for row in rows {
            let (id, name, location, raw_status, model_id) = row?;

            // Convert SQL text into a validated domain status.
            let status = raw_status.parse::<SensorStatus>()?;

            let sensor = Sensor::restore(
                id, name, location, status, model_id,
            )?;

            sensors.push(sensor);
        }

        Ok(sensors)
    }
}
