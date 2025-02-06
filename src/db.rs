use poise::serenity_prelude::prelude::TypeMapKey;
use sqlx::AnyConnection;
use sqlx::SqliteConnection;
use std::collections::HashMap;

type Error = crate::Error;
type Context<'a> = crate::Context<'a>;

pub struct DbConnection;
impl TypeMapKey for DbConnection {
    type Value = SqliteConnection;
}

pub struct ServerAddress;
impl TypeMapKey for ServerAddress {
    type Value = HashMap<String, String>;
}

pub async fn store_server_address(
    conn: &mut SqliteConnection,
    data: HashMap<String, String>,
) -> Result<(), Error> {
    for (key, value) in data {
        _ = sqlx::query!(
            "INSERT INTO server_address (key, value) VALUES (?, ?)
ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            key,
            value
        )
        .execute(&mut *conn)
        .await;
    }

    Ok(())
}

pub async fn read_server_address(
    conn: &mut SqliteConnection,
) -> Result<HashMap<String, String>, Error> {
    let rows = sqlx::query!("SELECT key, value FROM server_address")
        .fetch_all(conn)
        .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| Some((row.key?, row.value)))
        .collect())
}

pub struct Settings {
    external_redirector_address: String,
}
impl TypeMapKey for Settings {
    type Value = Settings;
}
