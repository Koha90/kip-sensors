use std::env;

use kip_sensors::catalog::SensorCatalog;
use kip_sensors::cli::{self, Command};
use kip_sensors::sensor::Sensor;
use kip_sensors::storage::SqliteSensorStore;

fn main() {
    let mut catalog = SensorCatalog::new();

    let sensors = [
        Sensor::new(
            1,
            "PT-101".to_string(),
            "Пастеризатор, линия 1".to_string(),
        ),
        Sensor::new(
            2,
            "PT-202".to_string(),
            "Танк хранения".to_string(),
        ),
        Sensor::new(
            3,
            "PT-303".to_string(),
            "Приёмный резервуар".to_string(),
        ),
    ];

    for sensor in sensors {
        match sensor {
            Ok(sensor) => {
                if let Err(err) = catalog.add(sensor) {
                    println!("Ошибка каталога: {}", err);
                }
            }
            Err(err) => {
                println!("Ошибка датчика: {}", err);
            }
        }
    }

    let args: Vec<String> = env::args().collect();

    match cli::parse(&args) {
        Ok(Command::List) => {
            for sensor in catalog.sensors() {
                println!(
                    "{:>3} | {:<10} | {:<30} | {:<12}",
                    sensor.id(),
                    sensor.name(),
                    sensor.location(),
                    sensor.status(),
                );
            }
        }

        Ok(Command::Show(id)) => match catalog.find(id) {
            Some(sensor) => {
                println!("ID: {}", sensor.id());
                println!("Датчик: {}", sensor.name());
                println!(
                    "Расположение: {}",
                    sensor.location()
                );
                println!("Статус: {}", sensor.status());
            }

            None => {
                println!("Дачик с ID {} не найден", id);
            }
        },
        Err(err) => {
            println!("Ошибка: {}", err);
        }

        Ok(Command::Remove(id)) => match catalog.remove(id) {
            Some(sensor) => {
                println!(
                    "Датчик {} с ID {} удалён",
                    sensor.name(),
                    sensor.id()
                );
            }

            None => {
                println!("Датчик с ID {} не найден", id)
            }
        },

        Ok(Command::Add { id, name, location }) => match Sensor::new(id, name, location) {
            Ok(sensor) => {
                if let Err(err) = catalog.add(sensor) {
                    println!("Ошибка каталога: {}", err);
                } else {
                    println!("Датчик добавлен");
                }
            }

            Err(err) => {
                println!("Ошибка датчика: {}", err);
            }
        },
    }

    let _store =
        SqliteSensorStore::open("sensors.db").expect("failed to initialize sensor database");
}
