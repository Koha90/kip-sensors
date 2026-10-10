//! SQLite persistence for the sensor catalog.

use rusqlite::{Connection, Result};

/// Manages the SQLite database used by the sensor catalog.
pub struct SqliteSensorStore {
    conn: Connection,
}

impl SqliteSensorStore {
    /// Opens a SQLite database and initializes its schema.
    ///
    /// Creates the database file if it does not exist.
    /// Use `:memory:` to create a temporary in-memory database.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened
    /// or its schema cannot be initialized.
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Embed the schema at compile time and execute it.
        conn.execute_batch(include_str!(
            "../migrations/001_create_sensors.sql"
        ))?;

        Ok(Self { conn })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_creates_sensors_table() -> Result<()> {
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
}
