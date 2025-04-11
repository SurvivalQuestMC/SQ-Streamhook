use std::env;

use rand::{Rng, distr::Alphanumeric};
use reqwest::header;
use serde::Deserialize;

use crate::{
    CLIENT_ID, CLIENT_SECRET, Url, build_url,
    database::{
        retrieve_app_auth_token, retrieve_user_access_token, retrieve_user_refresh_token,
        store_app_auth_token, store_user_auth_tokens,
    },
    server::streamhook_server,
};

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct AppAccessToken {
    pub access_token: String,
    expires_in: u32,
    token_type: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct UserAccessToken {
    pub access_token: String,
    expires_in: u32,
    refresh_token: String,
    scope: Vec<String>,
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
    client: &reqwest::Client,
) -> anyhow::Result<()> {
    let res = validate_streamhook(conn, client.clone()).await?;
    if !res {
        println!("Auth Token is invalid, generating new token.");
        let token = authenticate_streamhook(client).await?;
        store_app_auth_token(conn, token.access_token).await?;
    };
    Ok(())
}

pub async fn validate_streamhook(
    conn: &mut sqlx::SqliteConnection,
    client: reqwest::Client,
) -> anyhow::Result<bool> {
    let auth_token = retrieve_app_auth_token(conn).await;
    if auth_token.is_none() {
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

async fn authenticate_streamhook(client: &reqwest::Client) -> anyhow::Result<AppAccessToken> {
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

    let deserialized: AppAccessToken = serde_json::from_str(&res[..]).unwrap();

    Ok(deserialized)
}

pub async fn refresh_user(
    conn: &mut sqlx::SqliteConnection,
    client: &reqwest::Client,
) -> anyhow::Result<()> {
    let res = validate_user(conn, &client).await?;
    if !res {
        println!("User Token is invalid, generating new one");
        let mut token = authenticate_user_refresh_token(conn, &client).await?;
        if token.is_none() {
            println!("Refresh Token is invalid, please reauthenticate the chat bot");
            token = Some(authenticate_user(&client).await?);
        }
        let token = token.unwrap();
        store_user_auth_tokens(conn, token.access_token, token.refresh_token).await?;
    }
    Ok(())
}

pub async fn validate_user(
    conn: &mut sqlx::SqliteConnection,
    client: &reqwest::Client,
) -> anyhow::Result<bool> {
    let auth_token = retrieve_user_access_token(conn).await;
    if auth_token.is_none() {
        return Ok(false);
    };

    let header_auth_string = format!("OAuth {}", auth_token.unwrap()).parse()?;

    println!("Validating User Token");

    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", header_auth_string);

    let res = client
        .get("https://id.twitch.tv/oauth2/validate")
        .headers(headers)
        .send()
        .await?
        .status();

    if res.is_success() {
        println!("User Token is valid!");
        return Ok(true);
    };

    Ok(false)
}

async fn authenticate_user_refresh_token(
    conn: &mut sqlx::SqliteConnection,
    client: &reqwest::Client,
) -> anyhow::Result<Option<UserAccessToken>> {
    let refresh_token = retrieve_user_refresh_token(conn).await;
    if refresh_token.is_none() {
        return Ok(None);
    }
    let refresh_token = refresh_token.unwrap();

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/x-www-form-urlencoded".parse()?);

    let (client_id, client_secret) = get_client_info();

    let res = client
        .post("https://id.twitch.tv/oauth2/token")
        .headers(headers)
        .body(format!("client_id={client_id}&client_secret={client_secret}&grant_type=refresh_token&refresh_token={refresh_token}"))
        .send()
        .await?
        .text()
        .await?;

    let deserialized: UserAccessToken = serde_json::from_str(&res[..]).unwrap();

    Ok(Some(deserialized))
}

async fn authenticate_user(client: &reqwest::Client) -> anyhow::Result<UserAccessToken> {
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
            "&scope=user%3Aread%3Achat+user%3Awrite%3Achat+user%3Abot+moderator%3Aread%3Achatters".into(),
            format!("&state={state}"),
        ]),
    });

    println!("Authorize Twitch Account");
    println!("{auth_code_path}");
    let code = streamhook_server(state.clone()).await;

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/x-www-form-urlencoded".parse()?);

    let (client_id, client_secret) = get_client_info();

    let redirect_uri = "http://localhost:3000";

    let res = client
        .post("https://id.twitch.tv/oauth2/token")
        .headers(headers)
        .body(format!(
            "client_id={client_id}&client_secret={client_secret}&code={code}&grant_type=authorization_code&redirect_uri={redirect_uri}"
        ))
        .send()
        .await?
        .text()
        .await?;

    let deserialized: UserAccessToken = serde_json::from_str(&res[..]).unwrap();
    println!("{deserialized:#?}");
    Ok(deserialized)
}
