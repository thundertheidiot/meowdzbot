use poise::serenity_prelude::prelude::TypeMapKey;
use serenity::prelude::TypeMap;
use sqlx::AnyConnection;
use sqlx::Connection;
use sqlx::Database;
use sqlx::Executor;
use sqlx::SqliteConnection;
use tokio::sync::RwLockReadGuard;
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

impl Default for Settings {
    fn default() -> Self {
	Settings {
	    external_redirector_address: "http://localhost:8080".to_string(),
	}
    }
}

// pub async fn read_settings<'a, DB, C>(
//     mut data: RwLockReadGuard<'_, TypeMap>,
//     conn: &mut SqliteConnection,
// ) -> Result<(), Error>
// {
//     let rows = sqlx::query!("SELECT external_redirector_address FROM settings WHERE id = 1").fetch_one(conn).await?;

//     let settings = Settings {
// 	external_redirector_address: rows.external_redirector_address.ok_or("DBError: Unable to get external redirector address")?,
//     };

//     data.insert::<Settings>(settings);
    
//     Ok(())
// }
