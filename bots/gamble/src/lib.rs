use std::{error::Error, sync::Arc, time::Duration};
use tracing::{info, warn, error};
use twilight_cache_inmemory::{DefaultInMemoryCache, ResourceType};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};
use twilight_http::Client as Http;

pub struct GreeterBot;

impl GreeterBot {
    /// Runs until Ctrl-C or fatal unrecoverable error.
    pub async fn run(token: String) -> Result<(), Box<dyn Error + Send + Sync>> {
        let intents = Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES | Intents::MESSAGE_CONTENT;

        // Simple supervisor loop: if the shard stream ends, recreate it after a short delay.
        loop {
            info!("greeter: starting shard");
            let http = Arc::new(Http::new(token.clone()));
            let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

            let cache = DefaultInMemoryCache::builder()
                .resource_types(ResourceType::MESSAGE)
                .build();

            while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
                let event = match item {
                    Ok(ev) => ev,
                    Err(err) => {
                        warn!(?err, "greeter: error receiving event; continuing");
                        continue;
                    }
                };

                cache.update(&event);

                if let Event::MessageCreate(msg) = event {
                    if msg.author.bot { continue; }
                    if msg.content.trim().eq_ignore_ascii_case("!helloworld") {
                        if let Err(why) = http.create_message(msg.channel_id)
                            .content("Hello, world!")
                            .await
                        {
                            error!(?why, "greeter: failed to send message");
                        }
                    }
                }
            }

            // If we’re here, the event stream ended (None).
            warn!("gamble: shard stream ended; reconnecting after backoff...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}
