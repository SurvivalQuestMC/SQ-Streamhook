use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct StreamhookConfig {
    version: u8,
    twitch_bot_account_name: String,
    universal_drops: HashMap<String, StreamDrop>,
}

#[derive(Deserialize, Debug)]
pub struct StreamDrop {
    name: String,
}

impl StreamhookConfig {
    pub fn get_bot_account_name(&self) -> &String {
        &self.twitch_bot_account_name
    }
}

pub fn streamhook_config() -> anyhow::Result<StreamhookConfig> {
    if fs::exists("config.yml").is_err() {
        let sample = include_bytes!("../config.sample.yml");
        let mut config = File::create("config.yml")?;
        config.write_all(sample)?;
    }

    let config_file = File::open("config.yml")?;
    let config_values: StreamhookConfig = serde_yml::from_reader(config_file)?;
    
    Ok(config_values)
}
