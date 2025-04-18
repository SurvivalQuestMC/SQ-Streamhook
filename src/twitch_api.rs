use reqwest::header;
use serde_json::Value;
use std::env;

use crate::{
    CLIENT_ID, StreamhookApp, database::retrieve_user_access_token,
};

pub async fn helix_get_chatters(app: &mut StreamhookApp) -> anyhow::Result<()> {
    let user_token = retrieve_user_access_token(&mut app.conn).await;
    let client_id = env::var(CLIENT_ID).unwrap();

    let header_auth_string = format!("Bearer {}", user_token.unwrap()).parse()?;

    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", header_auth_string);
    headers.insert("Client-ID", client_id.parse()?);

    let user_name = app.config.get_bot_account_name().as_str();
    
    let moderator_id = helix_get_user_id(app, user_name.to_string()).await?;
    let broadcaster_id = moderator_id.clone();

    let res = app.client
        .get(format!("https://api.twitch.tv/helix/chat/chatters?broadcaster_id={broadcaster_id}&moderator_id={moderator_id}"))
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;
    println!("Chatters for RestedWicked:");
    let res_value: Value = serde_json::from_str(&res[..]).unwrap();
    for object in 0..res_value["total"].as_u64().unwrap() {
        println!(
            "{}",
            res_value["data"][object as usize]["user_login"]
                .as_str()
                .unwrap()
        );
    }
    Ok(())
}

pub async fn helix_get_user_id(app: &mut StreamhookApp, user: String) -> anyhow::Result<String> {
    let user_token = retrieve_user_access_token(&mut app.conn).await;
    let client_id = env::var(CLIENT_ID).unwrap();

    let header_auth_string = format!("Bearer {}", user_token.unwrap()).parse()?;

    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", header_auth_string);
    headers.insert("Client-ID", client_id.parse()?);

    let res = app
        .client
        .get(format!(
            "https://api.twitch.tv/helix/users?login_id={user}"
        ))
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let res_value: Value = serde_json::from_str(&res[..]).unwrap();
    let user_id = res_value["data"][0]["id"].as_str().unwrap();

    Ok(user_id.to_string())
}
