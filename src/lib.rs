use std::env;

use reqwest::{header, Error};
use serde::Deserialize;
use sqlx::{sqlite::SqliteConnectOptions, Connection, Row, SqliteConnection};

// Database

pub async fn init_database() -> Result<SqliteConnection, anyhow::Error> {
    let opts = SqliteConnectOptions::new()
        .filename("streamhooks.db")
        .create_if_missing(true);

    let mut conn = SqliteConnection::connect_with(&opts).await?;
    sqlx::migrate!().run(&mut conn).await?;
    Ok(conn)
}

pub async fn retrieve_auth_token(conn: &mut sqlx::SqliteConnection) -> Option<String> {
    let query = sqlx::query(
        r#"
SELECT access_token
FROM streamhooks_auth
LIMIT 1
        "#
    )
    .fetch_optional(conn)
    .await
    .unwrap();

    match query {
        None => return None,
        Some(row) => return row.try_get("access_token").unwrap(),
    };
}

pub async fn store_auth_token(conn: &mut sqlx::SqliteConnection, auth_token: String) {
    //let mut conn = conn.acquire().await.unwrap();

    println!("Clearing previous key..");
    sqlx::query(
        r#"
DELETE FROM streamhooks_auth
        "#
    )
    .execute(&mut *conn)
    .await
    .unwrap();
    println!("Previous key cleared!");

    println!("Inserting into database!");
    sqlx::query(
        r#"
INSERT INTO streamhooks_auth ( access_token )
VALUES ( ?1 )
        "#
    )
    .bind(auth_token)
    .execute(&mut *conn)
    .await
    .unwrap()
    .last_insert_rowid();
    println!("Succesfully inserted!");
}

// Authentication
const CLIENT_ID: &str = "STREAMHOOK_CLIENT_ID";
const CLIENT_SECRET: &str = "STREAMHOOK_CLIENT_SECRET";

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

pub async fn validate_auth_token(conn: &mut sqlx::SqliteConnection) -> Result<bool, Error> {
    let mut auth_token = retrieve_auth_token(conn).await;
    match auth_token {
        None => return Ok(false),
        Some(token) => auth_token = Some(token),
    };
    
    let header_auth_string = format!("OAuth {}", auth_token.unwrap()).parse().unwrap();

    println!("Validating Auth Token");

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Authorization",
        header_auth_string,
    );

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let res = client.get("https://id.twitch.tv/oauth2/validate")
        .headers(headers)
        .send()
        .await?
        .status();
    
    if res.is_success() == true {
        println!("Auth Token is valid!");
        return Ok(true)
    } else {
        println!("Auth Token is invalid, generating new token.");
        return Ok(false)
    }
}

pub fn authenticate_streamhooks() -> Result<OauthAccessToken, Error> {
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

    let deserialized: OauthAccessToken = serde_json::from_str(&res[..]).unwrap();

    Ok(deserialized)
}

pub fn authenticate_user() -> Result<OauthAccessToken, Error> {
    println!("Authorize Twitch Account");
    println!("https://id.twitch.tv/oauth2/authorize?response_type=token&client_id={}&redirect_uri=http://localhost:3000&scope=user%3Aread%3Achat+user%3Awrite%3Achat+user%3Abot&state=randomhere", env::var(CLIENT_ID).unwrap());
    todo!()
}
