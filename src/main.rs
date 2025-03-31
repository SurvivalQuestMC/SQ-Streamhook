use clap::Parser;
use sq_streamhook::{
    StreamhookMessage,
    auth::validate_auth_token,
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

            streamhook_update();
        }
    }

    Ok(())
}

async fn streamhook_init() -> anyhow::Result<()> {
    dotenvy::from_filename(".env").ok();
    let mut conn = init_database().await?;
    validate_auth_token(&mut conn).await?;

    Ok(())
}

fn streamhook_update() {
    todo!();
    //loop {

    //}
}
