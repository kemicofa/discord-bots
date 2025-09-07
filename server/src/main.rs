use std::env;
use common::init_tracing;
use tokio::{select, try_join};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
   dotenvy::dotenv().ok();
   init_tracing(); 

    // Each bot gets its own token (and thus its own shard & rate limits)
    // export DISCORD_TOKEN_GREETER=...; export DISCORD_TOKEN_MODERATOR=...
    let gamble_token = env::var("DISCORD_TOKEN_GAMBLE").map_err(|_| "Set DISCORD_TOKEN_GAMBLE")?;
    
    info!("starting bots (Ctrl-C to stop)");

    let bot_gamble = gamble::GreeterBot::run(gamble_token);

    select! {
        res = async {
            try_join!(
                bot_gamble,
            )
        } => { res?; }
        _ = tokio::signal::ctrl_c() => {
            eprintln!("Shutting down on Ctrl-C");
        }
    }

    Ok(())
}
