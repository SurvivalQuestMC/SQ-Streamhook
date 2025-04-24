use std::{
    fs::{self, File},
    io::Write,
    sync::OnceLock,
};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct StreamhookConfig {
    version: u8,
    mod_account: String,
    streamers: Vec<String>,
}

impl Default for StreamhookConfig {
    fn default() -> Self {
        if fs::exists("config.yml").is_err() {
            let sample = include_bytes!("../config.sample.yml");
            let mut config = File::create("config.yml").unwrap();
            config.write_all(sample).unwrap();
        }

        let config_file = File::open("config.yml").unwrap();
        let config_values: Self =
            serde_yml::from_reader(config_file).expect("Could not parse config");

        config_values
    }
}

impl StreamhookConfig {
    pub fn get<'a>() -> &'a Self {
        static CONFIG: OnceLock<StreamhookConfig> = OnceLock::new();
        CONFIG.get_or_init(|| Self::default())
    }

    pub fn get_mod_account() -> &'static String {
        &Self::get().mod_account
    }
}
