use std::fmt::Write;

pub mod auth;
pub mod cli;
pub mod database;
// App Data Types

pub enum StreamhookMessage {
    Stop,
    Start,
}

pub const CLIENT_ID: &str = "STREAMHOOK_CLIENT_ID";
pub const CLIENT_SECRET: &str = "STREAMHOOK_CLIENT_SECRET";

pub struct Url {
    protocol: String,
    domain: String,
    path: Option<String>,
    parameters: Option<Vec<String>>,
}

fn build_url(url: Url) -> String {
    let protocol = url.protocol;
    let domain = url.domain;
    let path = url.path;
    let parameters = url.parameters;

    let mut url = String::new();
    write!(&mut url, "{protocol}://{domain}/").unwrap();

    if let Some(path) = path {
        write!(&mut url, "{path}").unwrap();
    }
    if let Some(parameters) = parameters {
        for parameter in parameters {
            write!(&mut url, "{parameter}").unwrap();
        }
    }
    url
}
