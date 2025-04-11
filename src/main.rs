use std::time::Duration;

use clap::Parser;
use sq_streamhook::{
    StreamhookMessage,
    auth::{refresh_streamhook, refresh_user},
    cli::{Cli, streamhook_parse_args},
    database::init_database,
    twitch_api::helix_get_chatters,
};
use tokio::time;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    match streamhook_parse_args(args) {
        StreamhookMessage::Streamer => (),
        StreamhookMessage::Start => {
            let mut conn = init_database().await?;
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()?;

            streamhook_init(&mut conn, &client).await?;

            streamhook_update(&mut conn, &client).await?;
        }
    };

    Ok(())
}

async fn streamhook_init(
    conn: &mut sqlx::SqliteConnection,
    client: &reqwest::Client,
) -> anyhow::Result<()> {
    dotenvy::from_filename(".env").ok();

    refresh_streamhook(conn, client).await?;
    refresh_user(conn, client).await?;
    Ok(())
}

async fn streamhook_update(
    conn: &mut sqlx::SqliteConnection,
    client: &reqwest::Client,
) -> anyhow::Result<()> {
    let mut interval = time::interval(Duration::from_secs(60 * 10));
    loop {
        interval.tick().await;
        helix_get_chatters(conn, client).await?;
        println!("Its been 10 minutes");
        break;
    }
    Ok(())
}
