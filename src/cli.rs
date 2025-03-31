use clap::{Parser, Subcommand, builder::styling::AnsiColor};

use crate::StreamhookMessage;

// Command Line Interface

/// SQ Streamhook
#[derive(Parser, Debug)]
#[command(name = "SQ Streamhook")]
#[command(version, about, long_about = None, styles=cli_styles())]
pub struct Cli {
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
    Start,
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

pub fn streamhook_parse_args(cli: Cli) -> StreamhookMessage {
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

fn cli_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .header(AnsiColor::Green.on_default().bold())
        .usage(AnsiColor::Green.on_default().bold())
        .literal(AnsiColor::Cyan.on_default().bold())
        .placeholder(AnsiColor::BrightWhite.on_default())
}
