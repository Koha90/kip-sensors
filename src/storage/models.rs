//! SQLite operations for sensor models.

use rusqlite::{OptionalExtension, params};

use crate::sensor_model::SensorModel;

use super::{SqliteSensorStore, StorageResult};

impl SqliteSensorStore {
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
