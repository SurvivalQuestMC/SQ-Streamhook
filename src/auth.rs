use std::env;

use rand::{Rng, distr::Alphanumeric};
use reqwest::header;
use serde::Deserialize;

use crate::{
    CLIENT_ID, CLIENT_SECRET, Url, build_url,
    database::{retrieve_app_auth_token, store_app_auth_token},
    server::streamhook_server,
};

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct OauthAccessToken {
    pub access_token: String,
    expires_in: u32,
    token_type: String,
}

fn get_client_info() -> (String, String) {
    (
        env::var(CLIENT_ID).unwrap(),
        env::var(CLIENT_SECRET).unwrap(),
    )
}

pub async fn refresh_streamhook(
    conn: &mut sqlx::SqliteConnection,
    client: reqwest::Client,
) -> anyhow::Result<()> {
    let res = validate_streamhook(conn, client.clone()).await?;
    if !res {
        println!("Auth Token is invalid, generating new token.");
        let token = authenticate_streamhook(client.clone()).await?;
        store_app_auth_token(conn, token.access_token).await?;
    };
    Ok(())
}

pub async fn validate_streamhook(
    conn: &mut sqlx::SqliteConnection,
    client: reqwest::Client,
) -> anyhow::Result<bool> {
    let auth_token = retrieve_app_auth_token(conn).await;
    if let None = auth_token {
        return Ok(false);
    };

    let header_auth_string = format!("OAuth {}", auth_token.unwrap()).parse()?;

    println!("Validating Auth Token");

    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", header_auth_string);

    let res = client
        .get("https://id.twitch.tv/oauth2/validate")
        .headers(headers)
        .send()
        .await?
        .status();

    if res.is_success() {
        println!("Auth Token is valid!");
        return Ok(true);
    };

    Ok(false)
}

pub async fn authenticate_streamhook(client: reqwest::Client) -> anyhow::Result<OauthAccessToken> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/x-www-form-urlencoded".parse()?);

    let (client_id, client_secret) = get_client_info();

    let res = client
        .post("https://id.twitch.tv/oauth2/token")
        .headers(headers)
        .body(format!(
            "client_id={}&client_secret={}&grant_type=client_credentials",
            client_id, client_secret
        ))
        .send()
        .await?
        .text()
        .await?;

    let deserialized: OauthAccessToken = serde_json::from_str(&res[..]).unwrap();

    Ok(deserialized)
}

pub async fn authenticate_user() -> anyhow::Result<OauthAccessToken> {
    let state: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    let auth_code_path = build_url(Url {
        protocol: "https".into(),
        domain: "id.twitch.tv".into(),
        path: Some("oauth2/authorize".into()),
        parameters: Some(vec![
            "?response_type=code".into(),
            format!("&client_id={}", env::var(CLIENT_ID).unwrap()),
            "&redirect_uri=http://localhost:3000".into(),
            "&scope=user%3Aread%3Achat+user%3Awrite%3Achat+user%3Abot".into(),
            format!("&state={state}"),
        ]),
    });

    println!("Authorize Twitch Account");
    println!("{auth_code_path}");
    let code = streamhook_server(state.clone()).await;
    println!("connection state: {state}");
    println!("user access code: {code}");
    todo!()
}
