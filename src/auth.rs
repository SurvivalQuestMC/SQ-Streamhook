use std::env;

use reqwest::{Error, header};
use serde::Deserialize;

use crate::{
    CLIENT_ID, CLIENT_SECRET, Url, build_url,
    database::{retrieve_auth_token, store_auth_token},
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

pub async fn validate_auth_token(conn: &mut sqlx::SqliteConnection) -> Result<(), Error> {
    let auth_token = retrieve_auth_token(conn).await;
    if let Some(token) = auth_token {
        let header_auth_string = format!("OAuth {}", token).parse().unwrap();

        println!("Validating Auth Token");

        let mut headers = header::HeaderMap::new();
        headers.insert("Authorization", header_auth_string);

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();

        let res = client
            .get("https://id.twitch.tv/oauth2/validate")
            .headers(headers)
            .send()
            .await?
            .status();

        if res.is_success() {
            println!("Auth Token is valid!");
            return Ok(());
        };
    };

    println!("Auth Token is invalid, generating new token.");
    let token = authenticate_streamhooks().unwrap();
    store_auth_token(conn, token.access_token).await;

    Ok(())
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
    let auth_code_path = build_url(Url {
        protocol: "https".into(),
        domain: "id.twitch.tv".into(),
        path: Some("oauth2/authorize".into()),
        parameters: Some(vec![
            "?response_type=code".into(),
            format!("&client_id={}", env::var(CLIENT_ID).unwrap()),
            "&redirect_uri=http://localhost:3000".into(),
            "&scope=user%3Aread%3Achat+user%3Awrite%3Achat+user%3Abot".into(),
            "&state=randomgohere".into(),
        ]),
    });

    println!("Authorize Twitch Account");
    println!("{auth_code_path}");
    todo!()
}
