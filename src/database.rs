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

async fn store_database_value(
    conn: &mut sqlx::SqliteConnection,
    value: String,
    column: &str,
    table: &str,
) -> anyhow::Result<()> {
    let query = format!("INSERT INTO {table } ({column}) VALUES ( $1 )");
    println!("{query}");
    sqlx::query(&query)
        .bind(value)
        .bind(column)
        .execute(conn)
        .await?;

    Ok(())
}

async fn clear_database_table(
    conn: &mut sqlx::SqliteConnection,
    table: &str,
) -> anyhow::Result<()> {
    let query = format!("DELETE FROM {table}");
    sqlx::query(&query).bind(table).execute(conn).await?;

    Ok(())
}

pub async fn retrieve_app_auth_token(conn: &mut sqlx::SqliteConnection) -> Option<String> {
    retrieve_database_value(conn, "access_token", "streamhooks_auth").await
}

pub async fn store_app_auth_token(
    conn: &mut sqlx::SqliteConnection,
    auth_token: String,
) -> anyhow::Result<()> {
    println!("Clearing previous key..");
    clear_database_table(conn, "streamhooks_auth").await?;
    println!("Previous key cleared!");

    println!("Inserting into database!");
    store_database_value(conn, auth_token, "access_token", "streamhooks_auth").await?;
    println!("Succesfully inserted!");

    Ok(())
}
