use std::env;
use reqwest::header;

use crate::{database::retrieve_user_access_token, CLIENT_ID};

pub async fn helix_get_chatters(
    conn: &mut sqlx::SqliteConnection,
    client: &reqwest::Client,
) -> anyhow::Result<()> {
    let user_token = retrieve_user_access_token(conn).await;
    let client_id = env::var(CLIENT_ID).unwrap();
    
    let header_auth_string = format!("Bearer {}", user_token.unwrap()).parse()?;

    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", header_auth_string);
    headers.insert("Client-ID", client_id.parse()?);
    
    let moderator_id = "0";
    let broadcaster_id = "0";

    let res = client
        .get(format!("https://api.twitch.tv/helix/chat/chatters?broadcaster_id={broadcaster_id}&moderator_id={moderator_id}"))
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    println!("{res}");
    Ok(())
}
