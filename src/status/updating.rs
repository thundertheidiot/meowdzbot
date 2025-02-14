use db::remove_updating_status_message;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use poise::{serenity_prelude as serenity, CreateReply};

use ::serenity::all::{CacheHttp, ChannelId, EditMessage, MessageId};
use serenity::prelude::TypeMapKey;
use sqlx::SqliteConnection;

use crate::settings::Settings;
use crate::{db::DbConnection, Context};
use crate::{db::ServerAddress, socket::ServerSocket, status::make_status_message, Error};

pub struct UpdatingStatusMessages;
impl TypeMapKey for UpdatingStatusMessages {
    type Value = Vec<(u64, u64, String)>;
}

pub async fn status_message_update_loop(ctx: Arc<serenity::Context>) {
    let mut interval = time::interval(Duration::from_secs(15));

    time::interval(Duration::from_secs(2)).tick().await;

    loop {
        interval.tick().await;

        let data = ctx.data.read().await;
        match data.get::<UpdatingStatusMessages>() {
            Some(v) => {
                for (c, m, name) in v.clone() {
                    let ctx = ctx.clone();

                    tokio::spawn(async move {
                        let http = ctx.http();

                        let ci = ChannelId::new(c);
                        let mi = MessageId::new(m);

                        match http.get_message(ci, mi).await {
                            Ok(mut msg) => {
                                let data = ctx.data.read().await;

                                let redirector: Option<String> = data
                                    .get::<Settings>()
                                    .and_then(|s| s.external_redirector_address.clone());

                                let address: Option<&str> = data
                                    .get::<ServerAddress>()
                                    .and_then(|v| v.get(&name).and_then(|v| Some(v.as_str())));

                                match data.get::<ServerSocket>() {
                                    Some(socks) => {
                                        match make_status_message(redirector, socks, &name, address)
                                            .await
                                        {
                                            Ok((embed, action)) => {
                                                if let Err(e) = msg
                                                    .edit(
                                                        http,
                                                        EditMessage::new()
                                                            .content("")
                                                            .embed(embed)
                                                            .components(action),
                                                    )
                                                    .await
                                                {
                                                    eprintln!("Error editing message: {e}");
                                                } else {
                                                    println!(
                                                        "updated usm {} {}",
                                                        msg.channel_id, msg.id
                                                    );
                                                }
                                            }
                                            Err(e) => eprintln!("Error making status: {e}"),
                                        }
                                    }
                                    None => eprintln!("DataError: Unable to get sockets"),
                                }
                            }
                            Err(e) => eprintln!("Unable to get message: {e}"),
                        }
                    });
                }

                ()
            }
            None => eprintln!("DataError: Unable to get usm"),
        }
    }
}

fn create_usm_help() -> String {
    "Create a status message that will get continuously updated.
This requires admin privileges."
        .into()
}

#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES",
    required_bot_permissions = "MANAGE_MESSAGES",
    category = "Updating status message",
    help_text_fn = "create_usm_help"
)]
pub async fn create_updating_status(
    ctx: Context<'_>,
    #[description = "Server identifier"] name: String,
) -> Result<(), Error> {
    ctx.send(
        CreateReply::default()
            .content("Message will be sent soon, feel free to dismiss this")
            .ephemeral(true),
    )
    .await?;

    let mut data = ctx.serenity_context().data.write().await;
    let usm = data
        .get_mut::<UpdatingStatusMessages>()
        .ok_or("DataError: Unable to get updating status messages")?;

    let channel = ctx.channel_id();
    let msg = channel
        .say(ctx, "Updating status message, please wait...")
        .await?;

    let entry: (u64, u64, String) = (msg.channel_id.into(), msg.id.into(), name);
    usm.push(entry.clone());

    let conn = data
        .get_mut::<DbConnection>()
        .ok_or("DataError: Unable to get database connection")?;

    db::add_updating_status_message(conn, entry).await?;

    Ok(())
}

fn delete_usm_help() -> String {
    "Delete an updating status message, and remove it from the database.
This requires admin privileges."
        .into()
}

#[poise::command(
    slash_command,
    context_menu_command = "Delete USM (admin only)",
    required_permissions = "MANAGE_MESSAGES",
    required_bot_permissions = "MANAGE_MESSAGES",
    category = "Updating status message",
    help_text_fn = "delete_usm_help"
)]
pub async fn delete_updating_status(
    ctx: Context<'_>,
    #[description = "Message"] message: serenity::Message,
) -> Result<(), Error> {
    let c = message.channel_id;
    let m = message.id;

    let mut data = ctx.serenity_context().data.write().await;

    {
        // Maybe weird, but sqlx doesn't throw an error if nothing is deleted, and this way i don't have to grab usms twice
        let conn = data
            .get_mut::<DbConnection>()
            .ok_or("DataError: Unable to get database connection")?;
        remove_updating_status_message(conn, c, m).await?;
    }

    let usms = data
        .get_mut::<UpdatingStatusMessages>()
        .ok_or("DataError: Unable to get updating status messages")?;
    if !usms
        .iter()
        .any(|(ci, mi, _name)| *ci == c.get() && *mi == m.get())
    {
        ctx.send(
            CreateReply::default()
                .content("This is not an updating status message")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    usms.retain(|(ci, mi, _name)| !(*ci == c.get() && *mi == m.get()));

    dbg!(usms);

    message.delete(ctx).await?;
    ctx.send(
        CreateReply::default()
            .content("Successfully deleted message")
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

pub mod db {
    use super::*;

    pub async fn read_updating_status_messages(
        conn: &mut SqliteConnection,
    ) -> Result<Vec<(u64, u64, String)>, Error> {
        struct Fetch {
            channel_id: String,
            message_id: String,
            server_name: String,
        }

        let usm = sqlx::query_as!(
            Fetch,
            "SELECT channel_id, message_id, server_name FROM status_messages"
        )
        .fetch_all(conn)
        .await?;

        Ok(usm
            .into_iter()
            .filter_map(|v| {
                Some((
                    v.channel_id.parse::<u64>().ok()?,
                    v.message_id.parse::<u64>().ok()?,
                    v.server_name,
                ))
            })
            .collect::<Vec<(u64, u64, String)>>())
    }

    pub async fn remove_updating_status_message(
        conn: &mut SqliteConnection,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<(), Error> {
        let c = channel_id.to_string();
        let m = message_id.to_string();

        sqlx::query!(
            "DELETE FROM status_messages WHERE channel_id = ? AND message_id = ?",
            c,
            m
        )
        .execute(conn)
        .await?;

        Ok(())
    }

    pub async fn add_updating_status_message(
        conn: &mut SqliteConnection,
        entry: (u64, u64, String),
    ) -> Result<(), Error> {
        let c = entry.0.to_string();
        let m = entry.1.to_string();
        let name = entry.2;

        sqlx::query!(
            "INSERT INTO status_messages (channel_id, message_id, server_name) VALUES (?, ?, ?)",
            c,
            m,
            name
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }
}
