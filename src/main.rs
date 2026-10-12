//! CLI entry point for the sensor catalog.

use std::env;
use std::process::ExitCode;

use kip_sensors::app_error::AppError;
use kip_sensors::cli::{self, Command};
use kip_sensors::sensor::Sensor;
use kip_sensors::storage::SqliteSensorStore;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Ошибка: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Executes a CLI command against persistent storage.
fn run() -> Result<(), AppError> {
    let args: Vec<String> = env::args().collect();

    // Validate user input before opening the database.
    let command = cli::parse(&args)?;

    // Open the persistent database.
    let store = SqliteSensorStore::open("sensors.db")?;

    match command {
        Command::List => {
            let sesnors = store.list()?;

            if sesnors.is_empty() {
                println!("Каталог датчиков пуст");
            }

            for sensor in sesnors {
                println!(
                    "{:>3} | {:<10} | {:<30} | {:12}",
                    sensor.id(),
                    sensor.name(),
                    sensor.location(),
                    sensor.status(),
                );
            }
        }

        Command::Show(id) => {
            let sensor = store
                .find(id)?
                .ok_or(AppError::NotFound(id))?;

            println!("ID: {}", sensor.id());
            println!("Датчик: {}", sensor.name());
            println!(
                "Расположение: {}",
                sensor.location()
            );
            println!("Статус: {}", sensor.status());
        }

        Command::Add { id, name, location } => {
            let sensor = Sensor::new(id, name, location)?;

            store.insert(&sensor)?;

            println!("Датчик {} добавлен", sensor.name());
        }

        Command::Remove(id) => {
            if store.remove(id)? {
                println!("Датчик с ID {id} удалён");
            } else {
                return Err(AppError::NotFound(id));
            }
        }
    }

    Ok(())
}
