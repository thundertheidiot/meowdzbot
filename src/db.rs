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

// pub async fn read_address(conn: &mut SqliteConnection) -> Option<String>{
//     let address = sqlx::query!(
// 	    "SELECT server_address FROM settings WHERE id = 1"
//     )
//         .fetch_one(conn)
//         .await
// 	.ok()?;

//     address.server_address
// }

// pub async fn write_address(conn: &mut SqliteConnection, address: String) -> Result<(), Error> {
//     sqlx::query!(
// 	"INSERT INTO settings (id, server_address) VALUES (1, ?)
// ON CONFLICT(id) DO UPDATE SET server_address = excluded.server_address",
// 	address
//     )
// 	.execute(conn)
// 	.await?;

//     Ok(())
// }
