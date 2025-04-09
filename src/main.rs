use clap::Parser;
use sq_streamhook::{
    StreamhookMessage,
    auth::{authenticate_user, refresh_streamhook},
    cli::{Cli, streamhook_parse_args},
    database::init_database,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    match streamhook_parse_args(args) {
        StreamhookMessage::Stop => (),
        StreamhookMessage::Start => {
            streamhook_init().await?;

            streamhook_update()
        }
        StreamhookMessage::Debug => (),
    }

    Ok(())
}

async fn streamhook_init() -> anyhow::Result<()> {
    dotenvy::from_filename(".env").ok();
    let mut conn = init_database().await?;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    refresh_streamhook(&mut conn, &client).await?;
    authenticate_user(client).await?;
    Ok(())
}

fn streamhook_update() {
    todo!();
    //loop {

    //}
}
