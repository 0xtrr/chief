mod engine;

use crate::engine::config::{load_config, DataSource};
use crate::engine::validation;
use crate::engine::validation::{
    validate_event, BlockedType, JsonDataSource, ValidationDataSource,
};
use nostr_sdk::Event;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process;
use std::time::Duration;
use tokio::io::{stdin, stdout, AsyncWriteExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_postgres::{Error as PGError, NoTls};
use crate::engine::ratelimit::RateLimit;

/// Represents a request from the relay
#[derive(Deserialize)]
struct Request {
    #[serde(rename = "type")]
    type_field: String,
    event: Event,
}

/// Represents the response we provide back to the relay
#[derive(Serialize)]
struct Response {
    id: String,
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load config
    let config = match load_config("/etc/chief/config.toml") {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load config: {:?}", e);
            process::exit(1);
        }
    };

    // Select either JSON file or DB datasource, configured in the config.toml file
    let data_source = if config.datasource_mode == DataSource::DB {
        // Set up a database as the datasource
        let (client, connection) = tokio_postgres::connect(
            format!(
                "host={} port={} user={} password={} dbname={}",
                config.database.host,
                config.database.port,
                config.database.user,
                config.database.password,
                config.database.dbname
            )
            .as_str(),
            NoTls,
        )
        .await?;

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Box::new(client) as Box<dyn validation::ValidationDataSource>
    } else {
        // Set up a JSON file as the datasource
        let json_data_source =
            JsonDataSource::new_from_file(config.json.file_path.as_str())?;
        Box::new(json_data_source) as Box<dyn ValidationDataSource>
    };

    let rate_limit_engine = RateLimit::new(
        config.filters.rate_limit.max_events,
        Duration::from_secs(config.filters.rate_limit.time_window as u64)
    );

    // Set up stdin and stdout handles
    let mut reader = BufReader::new(stdin()).lines();
    let mut writer = stdout();

    while let Some(line) = reader.next_line().await.unwrap() {
        // Deserialize request from strfry
        let req: Request = serde_json::from_str(&line)?;

        // Type is currently always "new", anything else is an error as per Strfry documentation
        if req.type_field != "new" {
            eprintln!("unexpected request type {}", req.type_field);
            continue;
        }

        // Build default response
        let mut res = Response {
            id: req.event.id.to_hex(),
            action: String::from("reject"),
            msg: Some(String::from("blocked")),
        };

        // Validates if the event should be persisted or not against a set of filters and modifies the response thereafter
        match validate_event(&*data_source, &req.event, &config.filters, &rate_limit_engine).await {
            Ok(Some((BlockedType::RateLimit, value))) => {
                res.msg = Some(String::from(
                    "rate limited"
                ));
                let blocked_pubkey = if let Some(val) = value {
                    val.to_owned()
                } else {
                    String::from("")
                };
                println!(
                    "[BLOCKED] public key {} got rate limited",
                    blocked_pubkey
                );
            }
            Ok(Some((BlockedType::Pubkey, value))) => {
                res.msg = Some(String::from(
                    "public key does not have permission to write to relay",
                ));
                let blocked_pubkey = if let Some(val) = value {
                    val.to_owned()
                } else {
                    String::from("")
                };
                println!(
                    "[BLOCKED] public key {} blocked from writing to relay",
                    blocked_pubkey
                );
            }
            Ok(Some((BlockedType::Kind, value))) => {
                res.msg = Some(String::from("event kind blocked by relay"));
                let blocked_kind = if let Some(kind) = value {
                    kind.to_owned()
                } else {
                    String::from("")
                };
                println!("kind {} not accepted", blocked_kind);
            }
            Ok(Some((BlockedType::Word, _))) => {
                println!("blacklisted word");
            }
            Ok(None) => {
                res.action = String::from("accept");
                res.msg = None;
            }
            Err(err) => {
                eprintln!("error validating event: {}", err)
            }
        }

        // Output result of event validation, this is picked up by strfry for further processing
        writer
            .write_all(serde_json::to_string(&res)?.as_bytes())
            .await
            .unwrap();
        writer.write_all(b"\n").await.unwrap();
    }

    Ok(())
}

// Handling the error in case the database query fails
struct MyPGError(PGError);

impl From<PGError> for MyPGError {
    fn from(error: PGError) -> Self {
        MyPGError(error)
    }
}

impl From<MyPGError> for Box<dyn Error> {
    fn from(error: MyPGError) -> Box<dyn Error> {
        Box::new(error.0)
    }
}
