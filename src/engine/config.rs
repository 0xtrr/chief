use std::fs;
use serde::Deserialize;
use tokio::io;

#[derive(Deserialize)]
pub struct Config {
    pub filters: FiltersConfig,
    pub database: DatabaseConfig,
}

#[derive(Copy, Clone, Deserialize)]
pub struct FiltersConfig {
    pub public_key: bool,
    pub public_key_filter_mode: FilterModeConfig,
    pub kind: bool,
    pub kind_filter_mode: FilterModeConfig,
    pub word: bool
}
#[derive(Copy, Clone, Deserialize, PartialEq, Debug)]
pub enum FilterModeConfig {
    Blacklist,
    Whitelist,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub dbname: String,
}

/// Load TOML config file
pub fn load_config(filename: &str) -> Result<Config, ConfigError> {
    let content = fs::read_to_string(filename)
        .map_err(ConfigError::ReadError)?;

    toml::from_str(&content)
        .map_err(ConfigError::ParseError)
}

#[derive(Debug)]
pub enum ConfigError {
    ReadError(io::Error),
    ParseError(toml::de::Error)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;
    use super::*;

    #[test]
    fn test_load_valid_config() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut config_path = PathBuf::from(manifest_dir);
        config_path.push("test_resources/valid_config.toml");
        let path = config_path.to_str().unwrap();

        let config = load_config(path).unwrap();

        assert_eq!(config.filters.public_key, true);
        assert_eq!(config.filters.public_key_filter_mode, FilterModeConfig::Whitelist);
        assert_eq!(config.filters.kind, true);
        assert_eq!(config.filters.kind_filter_mode, FilterModeConfig::Blacklist);
        assert_eq!(config.filters.word, false);

        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, "5432");
        assert_eq!(config.database.user, "chief");
        assert_eq!(config.database.password, "changeme");
        assert_eq!(config.database.dbname, "chief");
    }

    #[test]
    fn test_load_invalid_config_missing_filters() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut config_path = PathBuf::from(manifest_dir);
        config_path.push("test_resources/invalid_config_missing_filters.toml");
        let path = config_path.to_str().unwrap();

        let result = load_config(path);

        assert!(result.is_err());

        match result {
            Err(ConfigError::ParseError(_)) => (),
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_load_invalid_config_invalid_filter_mode() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut config_path = PathBuf::from(manifest_dir);
        config_path.push("test_resources/invalid_config_invalid_filter_mode.toml");
        let path = config_path.to_str().unwrap();

        let result = load_config(path);

        assert!(result.is_err());

        match result {
            Err(ConfigError::ParseError(_)) => (),
            _ => panic!("Unexpected error type"),
        }
    }
}