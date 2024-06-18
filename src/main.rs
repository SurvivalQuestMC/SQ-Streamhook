use reqwest::{header, Error};
use serde::Deserialize;
use sqlx::{sqlite::SqlitePool, Executor};
use std::env;
use tokio::task::spawn_blocking;

const CLIENT_ID: &str = "STREAMHOOK_CLIENT_ID";
const CLIENT_SECRET: &str = "STREAMHOOK_CLIENT_SECRET";
const DATABASE: &str = "DATABASE_URL";

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
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
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::from_filename(".env").ok();

    let pool = SqlitePool::connect(&env::var(DATABASE).unwrap()).await?;
    let mut conn = pool.acquire().await?;

    conn.execute("CREATE TABLE IF NOT EXISTS streamhooks_auth ( access_token TEXT )")
        .await?;

    validate_auth_token();

    let request = spawn_blocking(move || authenticate_streamhooks().unwrap())
        .await
        .unwrap();

    println!("Inserting into database!");
    let id = sqlx::query!(
        r#"
INSERT INTO streamhooks_auth ( access_token )
VALUES ( ?1 )
        "#,
        request.access_token
    )
    .execute(&mut *conn)
    .await?
    .last_insert_rowid();

    println!("Inserted at {id}");

    let queries = sqlx::query!(
        r#"
SELECT access_token
FROM streamhooks_auth
        "#
    )
    .fetch_all(&pool)
    .await?;

    for query in queries {
        println!("Key: {}", query.access_token.unwrap());
    }

    Ok(())
}

fn validate_auth_token() {
    todo!()
}

fn authenticate_streamhooks() -> Result<OauthAccessToken, Error> {
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
