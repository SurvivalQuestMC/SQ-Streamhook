use std::{
    fs::{self, File},
    io::Write,
    sync::{Mutex, MutexGuard, OnceLock},
};

use serde::Deserialize;

static CONFIG: OnceLock<Mutex<StreamhookConfig>> = OnceLock::new();

#[derive(Deserialize, Debug, Clone)]
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

        if config_values.version != 1 {
            panic!("Wrong Config Version Used");
        }

        config_values
    }
}

impl StreamhookConfig {
 
    fn get<'a>() -> &'a Mutex<Self> {
        CONFIG.get_or_init(|| Self::default().into())
    }

    pub fn reload() {
        let mut config = Self::get().lock().unwrap();
        *config = StreamhookConfig::default();
    }

    pub fn get_config() -> MutexGuard<'static, StreamhookConfig> {
        Self::get().lock().unwrap()
    }

    pub fn get_mod_account() -> String {
        Self::get().lock().unwrap().mod_account.to_string()
    }
}
