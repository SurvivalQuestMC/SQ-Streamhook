use std::time::Duration;

use clap::Parser;
use sq_streamhook::{
    StreamhookApp, StreamhookMessage,
    auth::{refresh_streamhook, refresh_user},
    cli::{Cli, streamhook_parse_args},
    config::StreamhookConfig,
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
            let mut app = streamhook_init().await?;

            streamhook_update(&mut app).await?;
        }
    };

    Ok(())
}

async fn streamhook_init() -> anyhow::Result<StreamhookApp> {
    dotenvy::from_filename(".env").ok();
    let conn = init_database().await?;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let mut app = StreamhookApp {
        conn,
        client,
    };

    refresh_streamhook(&mut app).await?;
    refresh_user(&mut app).await?;

    Ok(app)
}

async fn streamhook_update(app: &mut StreamhookApp) -> anyhow::Result<()> {
    let mut interval = time::interval(Duration::from_secs(60 * 10));
    loop {
        interval.tick().await;
        helix_get_chatters(app).await?;
        println!("Its been 10 minutes");
        break;
    }
    Ok(())
}
