use crate::settings::Settings;
use crate::Error;
use poise::serenity_prelude::prelude::TypeMapKey;
use sqlx::SqliteConnection;
use std::collections::HashMap;

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
        .await?;
    }

    Ok(())
}

pub async fn remove_server_address(
    conn: &mut SqliteConnection,
    name: &String,
) -> Result<(), Error> {
    sqlx::query!("DELETE FROM server_address WHERE key = ?", name)
        .execute(conn)
        .await?;

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

pub async fn store_settings(settings: Settings, conn: &mut SqliteConnection) -> Result<(), Error> {
    _ = sqlx::query!(
        "INSERT INTO settings (id, external_redirector_address) VALUES (1, ?)
ON CONFLICT(id) DO UPDATE SET external_redirector_address = excluded.external_redirector_address",
        settings.external_redirector_address
    )
    .execute(conn)
    .await?;

    Ok(())
}
