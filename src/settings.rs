use db::store_settings;
use crate::Error;
use poise::{serenity_prelude::prelude::TypeMapKey, CreateReply};

use crate::{db::DbConnection, Context};

#[derive(Clone)]
pub struct Settings {
    #[allow(unused)]
    id: i64, // The id field is in the database, to enforce only one row, this will always be 1
    pub external_redirector_address: Option<String>,
    pub activity_server_identifier: Option<String>,
    pub activity_server_max_players: Option<i64>,
}
impl TypeMapKey for Settings {
    type Value = Settings;
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            id: 1,
            external_redirector_address: Some("https://dz.kotiboksi.xyz".to_string()),
	    activity_server_identifier: Some("meow".into()),
	    activity_server_max_players: Some(16),
        }
    }
}

use crate::privilege_check;
#[poise::command(slash_command, check = "privilege_check")]
pub async fn set_external_redirector(
    ctx: Context<'_>,
    #[description = "External address"] addr: String,
) -> Result<(), Error> {
    let mut data = ctx.serenity_context().data.write().await;

    let msg = ctx
        .send(
            CreateReply::default()
                .content(format!("Updating address to {}...", &addr))
                .ephemeral(true),
        )
        .await?;

    let settings = data
        .get_mut::<Settings>()
        .ok_or("DataError: Unable to get settings")?;
    settings.external_redirector_address = Some(addr);

    let settings = settings.to_owned();

    let conn = data
        .get_mut::<DbConnection>()
        .ok_or("DataError: Unable to get database connection")?;
    store_settings(settings, conn).await?;

    msg.edit(
        ctx,
        CreateReply::default()
            .content("Redirector address successfully updated")
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

pub mod db {
    use super::Settings;
    use crate::Error;
    use serenity::prelude::TypeMap;
    use sqlx::SqliteConnection;
    use tokio::sync::RwLockWriteGuard;

    pub async fn read_settings(
        data: &mut RwLockWriteGuard<'_, TypeMap>,
        conn: &mut SqliteConnection,
    ) -> Result<(), Error> {
        let settings = sqlx::query_as!(Settings, "SELECT * FROM settings WHERE id = 1")
            .fetch_one(conn)
            .await
            .unwrap_or_default();

        data.insert::<Settings>(settings);

        Ok(())
    }

    pub async fn store_settings(settings: Settings, conn: &mut SqliteConnection) -> Result<(), Error> {
    _ = sqlx::query!(
        "INSERT INTO settings (
 id,
 external_redirector_address,
 activity_server_identifier,
 activity_server_max_players
) VALUES (1, ?, ?, ?)
ON CONFLICT(id) DO UPDATE
SET external_redirector_address = excluded.external_redirector_address,
    activity_server_identifier = excluded.activity_server_identifier,
    activity_server_max_players = excluded.activity_server_max_players",
        settings.external_redirector_address,
	settings.activity_server_identifier,
	settings.activity_server_max_players,
    )
    .execute(conn)
    .await?;

    Ok(())
}
}
