use serde::Deserialize;
use std::fmt::Formatter;
use std::fs;
use tokio::io;

#[derive(Deserialize)]
pub struct Config {
    pub datasource_mode: DataSource,
    pub filters: FiltersConfig,
    pub database: DatabaseDatasourceConfig,
    pub json: JsonDatasourceConfig,
}

#[derive(Deserialize, PartialEq, Debug)]
pub enum DataSource {
    Json,
    Db,
}

#[derive(Clone, Deserialize)]
pub struct FiltersConfig {
    pub pubkey: PubkeyFilterConfig,
    pub kind: KindFilterConfig,
    pub content: ContentFilterConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Clone, Deserialize, PartialEq, Debug)]
pub enum FilterModeConfig {
    Blacklist,
    Whitelist,
}

#[derive(Clone, Deserialize)]
pub struct PubkeyFilterConfig {
    pub enabled: bool,
    pub filter_mode: FilterModeConfig,
}

#[derive(Clone, Deserialize)]
pub struct KindFilterConfig {
    pub enabled: bool,
    pub filter_mode: FilterModeConfig,
}

#[derive(Clone, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub max_events: u32,
    pub time_window: u32, // in seconds
}

#[derive(Clone, Deserialize)]
pub struct ContentFilterConfig {
    pub enabled: bool,
    pub validated_kinds: Vec<u32>,
}

#[derive(Deserialize)]
pub struct DatabaseDatasourceConfig {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub dbname: String,
}

#[derive(Deserialize)]
pub struct JsonDatasourceConfig {
    pub file_path: String,
}

/// Load TOML config file
pub fn load_config(filename: &str) -> Result<Config, ConfigError> {
    let content = fs::read_to_string(filename).map_err(ConfigError::ReadError)?;

    toml::from_str(&content).map_err(ConfigError::ParseError)
}

#[derive(Debug)]
pub enum ConfigError {
    ReadError(io::Error),
    ParseError(toml::de::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ReadError(e) => write!(f, "Error reading configuration file: {}", e),
            ConfigError::ParseError(e) => write!(f, "Error parsing configuration file: {}", e),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::validation::JsonDataSource;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn test_load_valid_config() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut config_path = PathBuf::from(manifest_dir);
        config_path.push("test_resources/valid_config_postgres.toml");
        let path = config_path.to_str().unwrap();

        let config = load_config(path).unwrap();

        assert_eq!(config.datasource_mode, DataSource::Db);

        assert_eq!(config.filters.pubkey.enabled, true);
        assert_eq!(config.filters.pubkey.filter_mode, FilterModeConfig::Whitelist);

        assert_eq!(config.filters.kind.enabled, true);
        assert_eq!(config.filters.kind.filter_mode, FilterModeConfig::Blacklist);

        assert_eq!(config.filters.rate_limit.enabled, false);
        assert_eq!(config.filters.rate_limit.max_events, 10);
        assert_eq!(config.filters.rate_limit.time_window, 60);

        assert_eq!(config.filters.content.enabled, false);
        assert_eq!(config.filters.content.validated_kinds, [1]);

        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, "5432");
        assert_eq!(config.database.user, "chief");
        assert_eq!(config.database.password, "changeme");
        assert_eq!(config.database.dbname, "chief");

        assert_eq!(config.json.file_path, "");
    }

    #[test]
    fn test_load_valid_config_json_mode() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut config_path = PathBuf::from(manifest_dir);
        config_path.push("test_resources/valid_config_json_mode.toml");
        let path = config_path.to_str().unwrap();

        let config = load_config(path).unwrap();

        assert_eq!(config.datasource_mode, DataSource::Json);

        assert_eq!(config.filters.pubkey.enabled, true);
        assert_eq!(config.filters.pubkey.filter_mode, FilterModeConfig::Whitelist);

        assert_eq!(config.filters.kind.enabled, true);
        assert_eq!(config.filters.kind.filter_mode, FilterModeConfig::Blacklist);

        assert_eq!(config.filters.rate_limit.enabled, false);
        assert_eq!(config.filters.rate_limit.max_events, 10);
        assert_eq!(config.filters.rate_limit.time_window, 60);

        assert_eq!(config.filters.content.enabled, false);
        assert_eq!(config.filters.content.validated_kinds, [1]);

        assert_eq!(config.database.host, "");
        assert_eq!(config.database.port, "");
        assert_eq!(config.database.user, "");
        assert_eq!(config.database.password, "");
        assert_eq!(config.database.dbname, "");

        assert_eq!(config.json.file_path, "/etc/chief/data.json");
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

    #[test]
    fn test_json_datasource() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut config_path = PathBuf::from(manifest_dir);
        config_path.push("test_resources/data.json");
        let path = config_path.to_str().unwrap();
        let json_datasource = JsonDataSource::new_from_file(path).unwrap();

        assert_eq!(json_datasource.pubkeys.len(), 1);
        assert_eq!(
            json_datasource.pubkeys.first().unwrap(),
            "d30effaa4af9d1522381866487bb0009203d687d44278dea3826be1ea64c46a8"
        );

        assert_eq!(json_datasource.kinds.len(), 1);
        assert_eq!(json_datasource.kinds.first().unwrap(), &1u32);

        assert_eq!(json_datasource.words.len(), 1);
        assert_eq!(json_datasource.words.first().unwrap(), "etf");
    }
}
