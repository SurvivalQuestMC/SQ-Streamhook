use sqlx::{Connection, Row, SqliteConnection, sqlite::SqliteConnectOptions};

pub async fn init_database() -> Result<SqliteConnection, anyhow::Error> {
    let opts = SqliteConnectOptions::new()
        .filename("streamhooks.db")
        .create_if_missing(true);

    let mut conn = SqliteConnection::connect_with(&opts).await?;
    sqlx::migrate!().run(&mut conn).await?;
    Ok(conn)
}

async fn retrieve_database_value(
    conn: &mut sqlx::SqliteConnection,
    column: &str,
    table: &str,
) -> Option<String> {
    let query = format!("SELECT {column} FROM {table} LIMIT 1");
    let result = sqlx::query(&query).fetch_optional(conn).await.unwrap();

    match result {
        None => None,
        Some(row) => row.try_get(column).unwrap(),
    }
}

async fn clear_database_table(
    conn: &mut sqlx::SqliteConnection,
    table: &str,
) -> anyhow::Result<()> {
    let query = format!("DELETE FROM {table}");
    sqlx::query(&query).bind(table).execute(conn).await?;

    Ok(())
}

async fn store_app_database_value(
    conn: &mut sqlx::SqliteConnection,
    access_token: String,
) -> anyhow::Result<()> {
    let query = "INSERT INTO streamhooks_app_auth (access_token) VALUES ( $1 )";
    sqlx::query(query).bind(access_token).execute(conn).await?;

    Ok(())
}

async fn store_user_database_value(
    conn: &mut sqlx::SqliteConnection,
    access_token: String,
    refresh_token: String,
) -> anyhow::Result<()> {
    let query = "INSERT INTO streamhooks_user_auth (access_token, refresh_token) VALUES ( $1, $2 )";
    sqlx::query(query)
        .bind(access_token)
        .bind(refresh_token)
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn retrieve_app_auth_token(conn: &mut sqlx::SqliteConnection) -> Option<String> {
    retrieve_database_value(conn, "access_token", "streamhooks_app_auth").await
}

pub async fn store_app_auth_token(
    conn: &mut sqlx::SqliteConnection,
    access_token: String,
) -> anyhow::Result<()> {
    println!("Clearing previous key..");
    clear_database_table(conn, "streamhooks_app_auth").await?;
    println!("Previous key cleared!");

    println!("Inserting into database!");
    store_app_database_value(conn, access_token).await?;
    println!("Succesfully inserted!");

    Ok(())
}

pub async fn retrieve_user_access_token(conn: &mut sqlx::SqliteConnection) -> Option<String> {
    retrieve_database_value(conn, "access_token", "streamhooks_user_auth").await
}

pub async fn retrieve_user_refresh_token(conn: &mut sqlx::SqliteConnection) -> Option<String> {
    retrieve_database_value(conn, "refresh_token", "streamhooks_user_auth").await
}

pub async fn store_user_auth_tokens(
    conn: &mut sqlx::SqliteConnection,
    access_token: String,
    refresh_token: String,
) -> anyhow::Result<()> {
    println!("Clearing previous key..");
    clear_database_table(conn, "streamhooks_user_auth").await?;
    println!("Previous key cleared!");

    println!("Inserting into database!");
    store_user_database_value(conn, access_token, refresh_token).await?;
    println!("Succesfully inserted!");

    Ok(())
}
