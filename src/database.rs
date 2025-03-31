use sqlx::{Connection, Row, SqliteConnection, sqlite::SqliteConnectOptions};

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
        "#,
    )
    .fetch_optional(conn)
    .await
    .unwrap();

    match query {
        None => None,
        Some(row) => row.try_get("access_token").unwrap(),
    }
}

pub async fn store_auth_token(conn: &mut sqlx::SqliteConnection, auth_token: String) {
    println!("Clearing previous key..");
    sqlx::query(
        r#"
DELETE FROM streamhooks_auth
        "#,
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
        "#,
    )
    .bind(auth_token)
    .execute(&mut *conn)
    .await
    .unwrap()
    .last_insert_rowid();
    println!("Succesfully inserted!");
}
