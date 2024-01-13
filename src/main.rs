mod engine;

use nostr_sdk::Event;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process;
use tokio::io::{stdin, stdout, AsyncWriteExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_postgres::{Error as PGError, NoTls};
use crate::engine::config::load_config;
use crate::engine::validation::{BlockedType, validate_event};

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
    let config = match load_config("/etc/chief.toml") {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load config: {:?}", e);
            process::exit(1);
        }
    };

    // Connect to the database
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
        match validate_event(&client, &req.event, &config.filters).await {
            Ok(Some((BlockedType::Pubkey, value))) => {
                res.msg = Some(String::from("public key does not have permission to write to relay"));
                println!("public key {} blocked from writing to relay", value);
            }
            Ok(Some((BlockedType::Kind, value))) => {
                res.msg = Some(String::from("event kind blocked by relay"));
                println!("kind {} not accepted", value);
            }
            Ok(Some((BlockedType::Word, value))) => {
                println!("blacklisted word {}", value);
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
