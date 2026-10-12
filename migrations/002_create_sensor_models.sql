CREATE TABLE sensor_models (
    id INTEGER PRIMARY KEY,
    manufacturer TEXT NOT NULL,
    model TEXT NOT NULL,
    part_number TEXT
);

ALTER TABLE sensors
ADD COLUMN model_id INTEGER
    REFERENCES sensor_models(id);
