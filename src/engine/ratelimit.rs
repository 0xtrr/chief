use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use nostr_sdk::Event;
use tokio::sync::Mutex;

pub struct RateLimit {
    pub cache: Arc<Mutex<HashMap<String, (Instant, u32)>>>,
    pub max_events: u32,
    pub time_window: Duration,
}

impl RateLimit {
    pub fn new(max_events: u32, time_window: Duration) -> Self {
        RateLimit {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_events,
            time_window,
        }
    }

    pub async fn is_allowed(&self, event: &Event) -> bool {
        let mut cache = self.cache.lock().await;
        let pubkey = event.pubkey.to_string();
        let now = Instant::now();

        if let Some((timestamp, count)) = cache.get_mut(&pubkey) {
            if now - *timestamp > self.time_window {
                *timestamp = now;
                *count = 1;
                true
            } else if *count < self.max_events {
                *count += 1;
                true
            } else {
                false
            }
        } else {
            cache.insert(pubkey, (now, 1));
            true
        }
    }
}