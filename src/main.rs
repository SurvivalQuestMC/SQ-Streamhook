use reqwest::{header, Error};
use serde::Deserialize;
use std::env;
use tokio::task::spawn_blocking;

const CLIENT_ID: &str = "STREAMHOOK_CLIENT_ID";
const CLIENT_SECRET: &str = "STREAMHOOK_CLIENT_SECRET";

#[derive(Deserialize, Debug)]
struct OauthAccessToken {
    access_token: String,
    expires_in: u32,
    token_type: String,
}

fn get_client_info() -> (String, String) {
    (
        env::var(CLIENT_ID).unwrap(),
        env::var(CLIENT_SECRET).unwrap(),
    )
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::from_filename(".env").ok();

    let request = spawn_blocking(move || authenticate_streamhooks().unwrap())
        .await
        .unwrap();
    let deserialized: OauthAccessToken = serde_json::from_str(&request[..]).unwrap();
    println!("{:#?}", deserialized);

    Ok(())
}

fn authenticate_streamhooks() -> Result<String, Error> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let (client_id, client_secret) = get_client_info();

    let res = client
        .post("https://id.twitch.tv/oauth2/token")
        .headers(headers)
        .body(format!(
            "client_id={}&client_secret={}&grant_type=client_credentials",
            client_id, client_secret
        ))
        .send()?
        .text()?;

    Ok(res)
}
