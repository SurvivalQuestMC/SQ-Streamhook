use clap::{Parser, Subcommand, builder::styling::AnsiColor};
use sq_streamhook::{init_database, validate_auth_token};

/// SQ Streamhook
#[derive(Parser, Debug)]
#[command(name = "SQ Streamhook")]
#[command(version, about, long_about = None, styles=cli_styles())]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
#[command(disable_help_subcommand = true)]
enum Commands {
    /// Manage Streamers
    Streamer {
        #[command(subcommand)]
        command: StreamerCommands,
    },
    /// Start Streamhook
    Start
}

#[derive(Subcommand, Debug)]
enum StreamerCommands {
    /// Add Streamer's Twitch Handle
    Add { streamer: String },
    /// Remove Streamer's Twitch Handle
    Remove { streamer: String },
    /// List Streamers
    List,
}

enum StreamhookMessage {
    Stop,
    Start,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
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

async fn streamhook_init() -> Result<(), anyhow::Error> {
    dotenvy::from_filename(".env").ok();
    let mut conn = init_database().await?;
    validate_auth_token(&mut conn).await?;
    Ok(())
}

fn streamhook_parse_args(cli: Cli) -> StreamhookMessage {
    use Commands::*;
    match cli.command {
        Streamer {command} => { 
            parse_streamer_args(command);
            StreamhookMessage::Stop
        },
        Start => StreamhookMessage::Start,
    }
}

fn parse_streamer_args(streamer_cli: StreamerCommands) {
    use StreamerCommands::*;
    match streamer_cli {
        Add {streamer} => println!("{streamer}"),
        Remove {streamer} => println!("{streamer}"),
        List => println!("List"),
    }
}

fn streamhook_update() {
    todo!();
    //loop {

    //}
}

fn cli_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .header(AnsiColor::Green.on_default().bold())
        .usage(AnsiColor::Green.on_default().bold())
        .literal(AnsiColor::Cyan.on_default().bold())
        .placeholder(AnsiColor::BrightWhite.on_default())
}
