use crate::db::store_settings;
use crate::Error;
use poise::{serenity_prelude::prelude::TypeMapKey, CreateReply};

use crate::{db::DbConnection, Context};

#[derive(Clone)]
pub struct Settings {
    pub id: i64,
    pub external_redirector_address: Option<String>,
}
impl TypeMapKey for Settings {
    type Value = Settings;
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            id: 1,
            external_redirector_address: Some("https://dz.kotiboksi.xyz".to_string()),
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
