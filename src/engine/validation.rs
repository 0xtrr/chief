use crate::engine::config::{FilterModeConfig, FiltersConfig};
use crate::engine::ratelimit::RateLimit;
use nostr_sdk::Event;
use serde::Deserialize;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use tokio_postgres::Client;

#[derive(Debug)]
pub enum BlockedType {
    Pubkey,
    Kind,
    Word,
    RateLimit,
}

#[derive(Deserialize)]
pub struct JsonDataSource {
    pub pubkeys: Vec<String>,
    pub kinds: Vec<u32>,
    pub words: Vec<String>,
}

impl JsonDataSource {
    pub fn new_from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let data: JsonDataSource = serde_json::from_reader(reader)?;
        Ok(data)
    }
}

type ValidationResult = Result<bool, Box<dyn Error>>;
type ValidationFuture<'a> = Pin<Box<dyn Future<Output = ValidationResult> + Send + 'a>>;

pub trait ValidationDataSource {
    fn is_pubkey_allowed(
        &self,
        pubkey: &str,
        filter_mode: FilterModeConfig,
    ) -> ValidationFuture<'_>;
    fn is_kind_allowed(&self, kind: u32, filter_mode: FilterModeConfig) -> ValidationFuture<'_>;
    fn is_content_allowed(&self, content: &str) -> ValidationFuture<'_>;
}

impl ValidationDataSource for tokio_postgres::Client {
    fn is_pubkey_allowed(&self, pubkey: &str, filter_mode: FilterModeConfig) -> ValidationFuture {
        let pubkey = pubkey.to_owned();
        Box::pin(async move {
            let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[&pubkey];
            is_allowed(
                self,
                "SELECT publickey FROM public_keys WHERE publickey = $1",
                params,
                filter_mode,
            )
            .await
        })
    }

    fn is_kind_allowed(&self, kind: u32, filter_mode: FilterModeConfig) -> ValidationFuture {
        Box::pin(async move {
            // We have to cast the event kind u32 to i32 to make tokio_postgres happy
            let i32_kind = kind as i32;

            let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[&i32_kind];
            is_allowed(
                self,
                "SELECT kind FROM kinds WHERE kind = $1",
                params,
                filter_mode,
            )
            .await
        })
    }

    fn is_content_allowed(&self, content: &str) -> ValidationFuture {
        let content = content.to_owned();
        Box::pin(async move {
            let word_stmt = self
                .prepare("SELECT word FROM words WHERE $1 ILIKE '%' || word || '%'")
                .await?;

            let rows = self.query(&word_stmt, &[&content]).await?;

            // We don't use the generic is_allowed function here because we want to get the blacklisted words. We're also always
            // using the blacklist filter mode for this check and therefore don't need the whitelist check functionality.
            Ok(rows.is_empty())
        })
    }
}

impl ValidationDataSource for JsonDataSource {
    fn is_pubkey_allowed(&self, pubkey: &str, filter_mode: FilterModeConfig) -> ValidationFuture {
        let pubkey = pubkey.to_owned();

        Box::pin(async move {
            match filter_mode {
                FilterModeConfig::Blacklist => Ok(!self.pubkeys.contains(&pubkey.to_string())),
                FilterModeConfig::Whitelist => Ok(self.pubkeys.contains(&pubkey.to_string())),
            }
        })
    }

    fn is_kind_allowed(&self, kind: u32, filter_mode: FilterModeConfig) -> ValidationFuture {
        Box::pin(async move {
            match filter_mode {
                FilterModeConfig::Blacklist => Ok(!self.kinds.contains(&kind)),
                FilterModeConfig::Whitelist => Ok(self.kinds.contains(&kind)),
            }
        })
    }

    fn is_content_allowed(&self, content: &str) -> ValidationFuture {
        let content = content.to_owned();
        Box::pin(async move {
            let contains_blacklisted_word = self
                .words
                .iter()
                .any(|word| content.contains(word.as_str()));
            Ok(!contains_blacklisted_word)
        })
    }
}

/// Validates the event data against a set of selected filter strategies. Could be public key, kind and/or content validation.
pub async fn validate_event(
    data_source: &dyn ValidationDataSource,
    event: &Event,
    filters: &FiltersConfig,
    rate_limit: &RateLimit,
) -> Result<Option<BlockedType>, Box<dyn Error>> {
    if filters.rate_limit.enabled
        && filters.rate_limit.max_events > 0
        && !rate_limit.is_allowed(event).await
    {
        return Ok(Some(BlockedType::RateLimit));
    }

    // Check if public key validation is activated
    if filters.pubkey.enabled {
        let publickey_allowed = data_source
            .is_pubkey_allowed(
                event.pubkey.to_string().as_str(),
                filters.pubkey.filter_mode.to_owned(),
            )
            .await?;
        if !publickey_allowed {
            return Ok(Some(BlockedType::Pubkey));
        }
    }

    // Check if kinf validation is activated
    if filters.kind.enabled {
        let kind_allowed = data_source
            .is_kind_allowed(event.kind.as_u32(), filters.kind.filter_mode.to_owned())
            .await?;
        if !kind_allowed {
            return Ok(Some(BlockedType::Kind));
        }
    }

    // Check if content validation is activated
    if filters.content.enabled {
        // Validate content only if we get a match with the event kind or the validated kinds list is empty
        if filters.content.validated_kinds.contains(&event.kind.as_u32())
            || filters.content.validated_kinds.is_empty()
        {
            let content_allowed = data_source
                .is_content_allowed(event.content.as_str())
                .await?;
            if !content_allowed {
                return Ok(Some(BlockedType::Word));
            }
        }
    }

    Ok(None)
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
