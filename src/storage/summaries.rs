//! Read-only SQLite queries for sensor summaries.

use super::{SqliteSensorStore, StorageResult};

/// Read-only summary of a sensor and its model.
#[derive(Debug, PartialEq, Eq)]
pub struct SensorSummary {
    /// Sensor identifier.
    pub id: u32,

    /// Sensor name.
    pub name: String,

    /// Installation location.
    pub location: String,

    /// Manufacturer, if a model is assigned.
    pub manufacturer: Option<String>,

    /// Model name, if available.
    pub model: Option<String>,
}

impl SqliteSensorStore {
    /// Returns sensors with their model information.
    ///
    /// Sensors wtihout an assigned model are included.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn list_with_models(&self) -> StorageResult<Vec<SensorSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                s.id,
                s.name,
                s.location,
                m.manufacturer,
                m.model
             FROM sensors AS s
             LEFT JOIN sensor_models AS m
                ON s.model_id = m.id
             ORDER BY s.id;",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(SensorSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                location: row.get(2)?,
                manufacturer: row.get(3)?,
                model: row.get(4)?,
            })
        })?;

        let mut sensors = Vec::new();

        for row in rows {
            sensors.push(row?);
        }

        Ok(sensors)
    }
}
