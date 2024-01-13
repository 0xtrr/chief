use std::error::Error;
use nostr_sdk::Event;
use nostr_sdk::key::XOnlyPublicKey;
use tokio_postgres::Client;
use crate::engine::config::{FilterModeConfig, FiltersConfig};

#[derive(Debug)]
pub enum BlockedType {
    Pubkey,
    Kind,
    Word,
}

/// Validates the event data against a set of selected filter strategies. Could be public key, kind and/or content validation.
pub async fn validate_event(
    client: &tokio_postgres::Client,
    event: &Event,
    filters: &FiltersConfig,
) -> Result<Option<(BlockedType, String)>, Box<dyn Error>> {

    // Check if public key validation is activated
    if filters.public_key {
        let publickey_allowed = verify_publickey_allowed(client, event.pubkey, filters.public_key_filter_mode).await?;
        if !publickey_allowed {
            return Ok(Some((BlockedType::Pubkey, event.pubkey.to_string())))
        }
    }

    // Check if kinf validation is activated
    if filters.kind {
        let kind_allowed = verify_kind_allowed(client, event.kind.as_u32(), filters.kind_filter_mode).await?;
        if !kind_allowed {
            return Ok(Some((BlockedType::Kind, event.kind.to_string())))
        }
    }

    // Check if content validation is activated
    if filters.word {
        let (content_allowed, blacklisted_words) = verify_content_allowed(client, event.content.to_string()).await?;
        if !content_allowed {
            return Ok(Some((BlockedType::Word, blacklisted_words.unwrap())))
        }
    }

    Ok(None)
}
/// Verifies if a public key is allowed to write events to the relay.
async fn verify_publickey_allowed(client: &tokio_postgres::Client, public_key: XOnlyPublicKey, filter_mode: FilterModeConfig) -> Result<bool, Box<dyn Error>> {
    let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[&public_key.to_string()];
    is_allowed(
        client,
        "SELECT publickey FROM public_keys WHERE publickey = $1",
        params,
        filter_mode,
    ).await
}

/// Verifies if a event of a certain kind is allowed to be written to the relay
async fn verify_kind_allowed(client: &tokio_postgres::Client, kind: u32, filter_mode: FilterModeConfig) -> Result<bool, Box<dyn Error>> {
    // We have to cast the event kind u32 to i32 to make tokio_postgres happy
    let i32_kind = kind as i32;

    let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[&i32_kind];
    is_allowed(
        client,
        "SELECT kind FROM kinds WHERE kind = $1",
        params,
        filter_mode,
    ).await
}

/// Verifies that the content of the event doesn't contain any blacklisted words
async fn verify_content_allowed(client: &tokio_postgres::Client, word: String) -> Result<(bool, Option<String>), Box<dyn Error>> {
    let word_stmt = client
        .prepare("SELECT word FROM words WHERE $1 ILIKE '%' || word || '%'")
        .await?;

    let rows = client
        .query(&word_stmt, &[&word])
        .await?;

    // We don't use the generic is_allowed function here because we want to get the blacklisted words. We're also always
    // using the blacklist filter mode for this check and therefore don't need the whitelist check functionality.
    if rows.is_empty() {
        Ok((true, None))
    } else {
        // Content contains a blacklisted word
        let matched_words: Vec<String> = rows
            .iter()
            .map(|row| row.get::<_, String>("word"))
            .collect();

        let matched_string = matched_words.join(", ");
        Ok((false, Some(matched_string)))
    }
}

/// Generic function to query database for something and decide whether to accept or deny the event based on filter mode selected
async fn is_allowed(
    client: &Client,
    query: &str,
    params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    filter_mode: FilterModeConfig,
) -> Result<bool, Box<dyn Error>> {
    let stmt = client.prepare(query).await?;
    let rows = client.query(&stmt, params).await?;

    match filter_mode {
        FilterModeConfig::Blacklist => Ok(rows.is_empty()),
        FilterModeConfig::Whitelist => Ok(!rows.is_empty()),
    }
}