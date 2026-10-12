use std::fmt;

/// Represents a parsed CLI command.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// Lists all sensors.
    List,

    /// Shows a sensor by its ID.
    Show(u32),

    /// Removes a sensor by its ID.
    Remove(u32),

    /// Adds a new sensor to the catalog.
    Add {
        id: u32,
        name: String,
        location: String,
    },

    /// Searches sensors by name or installation location.
    Search(String),
}

/// Represents errors that may occur while parsing CLI arguments.
#[derive(Debug, PartialEq)]
pub enum CliError {
    /// No command was provided.
    MissingCommand,

    /// The command is not supported.
    UnknownCommand(String),

    /// A command requires a sensor ID, but it was not provided.
    MissingId,

    /// The sensor ID could not be parsed as `u32`.
    InvalidId(String),

    /// A command requires a sensor name, but it was not provided.
    MissingName,

    /// A command requires a sensor location, but it was not provided.
    MissingLocation,

    /// A search command was provided without a query.
    MissingQuery,

    /// A search query contains only whitespace.
    EmptyQuery,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::MissingCommand => {
                write!(f, "command is missing")
            }

            CliError::UnknownCommand(command) => {
                write!(f, "unknown command: {}", command)
            }

            CliError::MissingId => {
                write!(f, "sensor ID is missing")
            }

            CliError::InvalidId(id) => {
                write!(f, "invalid sensor ID: {}", id)
            }

            CliError::MissingName => {
                write!(f, "sensor name is missing")
            }

            CliError::MissingLocation => {
                write!(f, "sensor location is missing")
            }

            CliError::MissingQuery => {
                write!(f, "search query is missing")
            }

            CliError::EmptyQuery => {
                write!(f, "search query must not be empty")
            }
        }
    }
}

/// Parses command-line arguments into a typed command.
///
/// # Errors
///
/// Returns:
/// - [`CliError::MissingCommand`] if no command is provided.
/// - [`CliError::UnknownCommand`] if the command is not supported.
/// - [`CliError::MissingId`] if `show` is used without an ID.
/// - [`CliError::InvalidId`] if the ID is not a valid `u32`.
/// - [`CliError::MissingName`] if a required sensor name is missing.
/// - [`CliError::MissingLocation`] if a required sensor location is missing.
/// - [`CliError::MissingQuery`] if a required query is missing.
/// - [`CliError::EmptyQuery`] if a required query has only whitespace.
///
/// Expected arguments layouts:
/// - `kip-sensors list`
/// - `kip-sensors show <id>`
/// - `kip-sensors remove <id>`
/// - `kip-sensors add <id> <name> <location>`
/// - `kip-sensors search <query>`
pub fn parse(args: &[String]) -> Result<Command, CliError> {
    let command = args
        .get(1)
        .ok_or(CliError::MissingCommand)?;

    match command.as_str() {
        "list" => Ok(Command::List),

        "show" => {
            let id = parse_id(args)?;
            Ok(Command::Show(id))
        }

        "remove" => {
            let id = parse_id(args)?;
            Ok(Command::Remove(id))
        }

        "add" => {
            let id = parse_id(args)?;

            let name = args
                .get(3)
                .ok_or(CliError::MissingName)?
                .clone();

            let location = args
                .get(4)
                .ok_or(CliError::MissingLocation)?
                .clone();

            Ok(Command::Add { id, name, location })
        }

        "search" => {
            let query = args
                .get(2)
                .ok_or(CliError::MissingQuery)?;

            let query = query.trim();

            if query.is_empty() {
                return Err(CliError::EmptyQuery);
            }

            Ok(Command::Search(query.to_owned()))
        }

        command => Err(CliError::UnknownCommand(
            command.to_string(),
        )),
    }
}

/// Parses a sensor ID from a CLI argument.
///
/// # Errors
///
/// Returns:
/// - [`CliError::MissingId`] if the argument is missing.
/// - [`CliError::InvalidId`] if the value is not a valid `u32`.
fn parse_id(args: &[String]) -> Result<u32, CliError> {
    let id = args
        .get(2)
        .ok_or(CliError::MissingId)?;

    id.parse::<u32>()
        .map_err(|_| CliError::InvalidId(id.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_list_command() {
        let args = vec!["kip-sensors".to_string(), "list".to_string()];

        let command = parse(&args);

        assert_eq!(command, Ok(Command::List));
    }

    #[test]
    fn parse_show_command() {
        let args = vec![
            "kip-sensors".to_string(),
            "show".to_string(),
            "42".to_string(),
        ];

        let command = parse(&args);

        assert_eq!(command, Ok(Command::Show(42)));
    }

    #[test]
    fn parse_missing_command_returns_error() {
        let args = vec!["kip-sensors".to_string()];

        assert_eq!(
            parse(&args),
            Err(CliError::MissingCommand)
        );
    }

    #[test]
    fn parse_unknown_command_returns_error() {
        let args = vec!["kip-sensors".to_string(), "banana".to_string()];

        assert_eq!(
            parse(&args),
            Err(CliError::UnknownCommand(
                "banana".to_string()
            ))
        );
    }

    #[test]
    fn parse_show_without_id_returns_error() {
        let args = vec!["kip-sensors".to_string(), "show".to_string()];

        assert_eq!(
            parse(&args),
            Err(CliError::MissingId)
        );
    }

    #[test]
    fn parse_show_with_invalid_id_returns_error() {
        let args = vec![
            "kip-sensors".to_string(),
            "show".to_string(),
            "кот".to_string(),
        ];

        assert_eq!(
            parse(&args),
            Err(CliError::InvalidId(
                "кот".to_string()
            ))
        );
    }

    #[test]
    fn parse_remove_command() {
        let args = vec![
            "kip-sensors".to_string(),
            "remove".to_string(),
            "42".to_string(),
        ];

        let command = parse(&args);

        assert_eq!(command, Ok(Command::Remove(42)));
    }

    #[test]
    fn parse_remove_without_id_returns_error() {
        let args = vec!["kip-sensors".to_string(), "remove".to_string()];

        assert_eq!(
            parse(&args),
            Err(CliError::MissingId)
        );
    }

    #[test]
    fn parse_remove_with_invalid_id_returns_error() {
        let args = vec![
            "kip-sensors".to_string(),
            "remove".to_string(),
            "кот".to_string(),
        ];

        assert_eq!(
            parse(&args),
            Err(CliError::InvalidId(
                "кот".to_string()
            ))
        );
    }

    #[test]
    fn parse_add_command() {
        let args = vec![
            "kip-sensors".to_string(),
            "add".to_string(),
            "4".to_string(),
            "PT-404".to_string(),
            "Гомогенизатор".to_string(),
        ];

        let command = parse(&args);

        assert_eq!(
            command,
            Ok(Command::Add {
                id: 4,
                name: "PT-404".to_string(),
                location: "Гомогенизатор".to_string(),
            })
        );
    }

    #[test]
    fn parse_add_without_name_returns_error() {
        let args = vec![
            "kip-sensors".to_string(),
            "add".to_string(),
            "4".to_string(),
        ];

        assert_eq!(
            parse(&args),
            Err(CliError::MissingName)
        );
    }

    #[test]
    fn parse_add_without_location_returns_error() {
        let args = vec![
            "kip-sensors".to_string(),
            "add".to_string(),
            "4".to_string(),
            "PT-404".to_string(),
        ];

        assert_eq!(
            parse(&args),
            Err(CliError::MissingLocation)
        );
    }

    #[test]
    fn parse_search_command() {
        let args = vec![
            "kip-sensors".to_string(),
            "search".to_string(),
            " Пастеризатор  ".to_string(),
        ];

        assert_eq!(
            parse(&args),
            Ok(Command::Search(
                "Пастеризатор".to_string()
            ))
        );
    }

    #[test]
    fn parse_search_without_query_returns_error() {
        let args = vec!["kip-sensors".to_string(), "search".to_string()];

        assert_eq!(
            parse(&args),
            Err(CliError::MissingQuery)
        );
    }

    #[test]
    fn parse_search_with_blank_query_returns_error() {
        let args = vec![
            "kip-sensors".to_string(),
            "search".to_string(),
            "   ".to_string(),
        ];

        assert_eq!(
            parse(&args),
            Err(CliError::EmptyQuery)
        );
    }
}
