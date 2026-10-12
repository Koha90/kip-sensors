-- Initail schema for the sensor catalog.
CREATE TABLE IF NOT EXISTS sensors (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    location TEXT NOT NULL,
    status TEXT NOT NULL
        CHECK (
            status IN (
                'Active',
                'Maintenance',
                'Failed',
                'Offline'
            )
        )
);
