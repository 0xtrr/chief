use nostr_sdk::Event;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use tokio::io::{stdin, stdout, AsyncWriteExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_postgres::{Error as PGError, NoTls};

#[derive(Deserialize)]
struct Request {
    #[serde(rename = "type")]
    type_field: String,
    event: Event,
}

#[derive(Serialize)]
struct Response {
    id: String,
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    database: DatabaseConfig,
}

#[derive(Deserialize)]
struct DatabaseConfig {
    host: String,
    port: String,
    user: String,
    password: String,
    dbname: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = load_config("/etc/chief.toml");

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

    let mut reader = BufReader::new(stdin()).lines();
    let mut writer = stdout();

    while let Some(line) = reader.next_line().await.unwrap() {
        let req: Request = serde_json::from_str(&line)?;

        if req.type_field != "new" {
            eprintln!("unexpected request type {}", req.type_field);
            continue;
        }

        let mut res = Response {
            id: req.event.id.to_hex(),
            action: String::from("reject"),
            msg: Some(String::from("blocked")),
        };

        match is_blacklisted(&client, &req.event).await {
            Ok(Some((BlacklistType::Pubkey, value))) => {
                println!("blacklisted publickey {}", value);
            }
            Ok(Some((BlacklistType::Kind, value))) => {
                println!("blacklisted kind {}", value);
            }
            Ok(Some((BlacklistType::Word, value))) => {
                println!("blacklisted word {}", value);
            }
            Ok(None) => {
                res.action = String::from("accept");
                res.msg = None;
            }
            Err(err) => {
                eprintln!("error checking blacklist: {}", err)
            }
        }

        writer
            .write_all(serde_json::to_string(&res)?.as_bytes())
            .await
            .unwrap();
        writer.write_all(b"\n").await.unwrap();
    }

    Ok(())
}

#[derive(Debug)]
enum BlacklistType {
    Pubkey,
    Kind,
    Word,
}

async fn is_blacklisted(
    client: &tokio_postgres::Client,
    event: &Event,
) -> Result<Option<(BlacklistType, String)>, Box<dyn Error>> {
    // Prepare the SQL query for checking if a pubkey exists in the blacklisted_pubkeys
    let blacklisted_pubkey_stmt = client
        .prepare("SELECT pubkey FROM blacklisted_pubkeys WHERE pubkey = $1")
        .await?;
    let blacklisted_kind_stmt = client
        .prepare("SELECT kind FROM blacklisted_kinds WHERE kind = $1")
        .await?;
    let blacklisted_word_stmt = client
        .prepare("SELECT word FROM blacklisted_words WHERE $1 ILIKE '%' || word || '%'")
        .await?;

    // Check for blacklisted public key
    let rows = client
        .query(&blacklisted_pubkey_stmt, &[&event.pubkey.to_string()])
        .await?;

    if !rows.is_empty() {
        return Ok(Some((BlacklistType::Pubkey, event.pubkey.to_string())));
    }

    // Check for blacklisted kind

    // We have to cast the event kind u32 to i32 to make tokio_postgres happy
    let i32_kind = event.kind.as_u32() as i32;
    let rows = client.query(&blacklisted_kind_stmt, &[&i32_kind]).await?;

    if !rows.is_empty() {
        return Ok(Some((BlacklistType::Kind, event.kind.to_string())));
    }

    // Check for blacklisted word
    let rows = client
        .query(&blacklisted_word_stmt, &[&event.content.to_string()])
        .await?;

    if !rows.is_empty() {
        let matched_words: Vec<String> = rows
            .iter()
            .map(|row| row.get::<_, String>("word"))
            .collect();

        let matched_string = matched_words.join(", ");

        return Ok(Some((BlacklistType::Word, matched_string)));
    }

    Ok(None)
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

fn load_config(filename: &str) -> Config {
    let content = fs::read_to_string(filename).expect("Error reading the config file");
    toml::from_str(&content).expect("Error parsing the config file")
}
