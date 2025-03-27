use sq_streamhook::{authenticate_streamhooks, authenticate_user, init_database, store_auth_token, validate_auth_token};
use tokio::task::spawn_blocking;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::from_filename(".env").ok();
    
    let mut conn = init_database().await?;

    if validate_auth_token(&mut conn).await.unwrap() != true {
        let request = spawn_blocking(move || authenticate_streamhooks().unwrap())
            .await
            .unwrap();
        store_auth_token(&mut conn, request.access_token).await;
    }

    authenticate_user()?;

    Ok(())
}

