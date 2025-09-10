use std::{ error::Error, sync::{ Arc, Mutex }, time::Duration };
use tracing::{ info, warn, error };
use twilight_cache_inmemory::{ DefaultInMemoryCache, ResourceType };
use twilight_gateway::{ Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt as _ };
use twilight_http::Client as Http;

use crate::{
    gamble_game_manager::{ GambleGameManager },
    utils::{ map_game_error_to_discord_message, map_ggm_response_to_discord_message },
};

mod gamble_game;
mod gamble_classic;
mod error;
mod types;
mod gamble_game_manager;
mod utils;

pub struct GambleBot;

impl GambleBot {
    /// Runs until Ctrl-C or fatal unrecoverable error.
    pub async fn run(token: String) -> Result<(), Box<dyn Error + Send + Sync>> {
        let intents = Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES | Intents::MESSAGE_CONTENT;

        let manager: Arc<Mutex<GambleGameManager>> = Arc::new(
            Mutex::new(GambleGameManager::default())
        );

        // Simple supervisor loop: if the shard stream ends, recreate it after a short delay.
        loop {
            info!("gamble: starting shard");
            let http = Arc::new(Http::new(token.clone()));
            let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

            let cache = DefaultInMemoryCache::builder()
                .resource_types(ResourceType::MESSAGE)
                .build();

            while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
                let event = match item {
                    Ok(ev) => ev,
                    Err(err) => {
                        warn!(?err, "gamble: error receiving event; continuing");
                        continue;
                    }
                };

                cache.update(&event);

                if let Event::MessageCreate(msg) = event {
                    if msg.author.bot {
                        continue;
                    }

                    let message = msg.content.trim();

                    if !message.starts_with("g!") {
                        continue;
                    }

                    let parts = msg.content.trim().split_ascii_whitespace().collect::<Vec<&str>>();

                    let channel_id = msg.channel_id.to_string();

                    let mut game_manager = manager.lock().unwrap();

                    let user_id = msg.author.id.to_string();

                    let send_message = async |message: String| {
                        if
                            let Err(why) = http
                                .create_message(msg.channel_id)
                                .content(&message).await
                        {
                            error!(?why, "gamble: failed to send message");
                        }
                    };

                    match
                        game_manager.execute(
                            channel_id.clone(),
                            user_id.clone(),
                            parts[0],
                            parts[1..].to_vec()
                        )
                    {
                        Ok(response) => {
                            let message = map_ggm_response_to_discord_message(&user_id, response);
                            if message.is_none() {
                                continue;
                            }
                            send_message(message.unwrap()).await;
                        }
                        Err(e) => {
                            let message = map_game_error_to_discord_message(&user_id, e);
                            send_message(message).await;
                        }
                    }

                    match game_manager.tick(channel_id) {
                        Ok(response) => {
                            let message = map_ggm_response_to_discord_message(&user_id, response);
                            if message.is_none() {
                                continue;
                            }
                            send_message(message.unwrap()).await;
                        }
                        Err(e) => {
                            let message = map_game_error_to_discord_message(&user_id, e);
                            send_message(message).await;
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
