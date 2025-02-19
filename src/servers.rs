use crate::servers::db::remove_server;
use crate::ServerSocket;
use crate::db::DbConnection;
use crate::Context;
use crate::Error;
use crate::privilege_check;
use poise::serenity_prelude::prelude::TypeMapKey;
use poise::CreateReply;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Server {
    pub name: String,
    pub addr: String,
    pub max_player_count: i64, // this could be u8 but sqlite is dumb
    pub legacy: bool,
    pub allow_upload_required: bool,
}

pub struct Servers;
pub type ServersValue = HashMap<String, Server>; // yes wasteful
impl TypeMapKey for Servers {
    type Value = ServersValue;
}

fn create_server_help() -> String {
    "Adds a new server to the list, counterintuitively this also works for updating the address of an existing server.
Requires admin privileges."
        .into()
}

#[poise::command(
    slash_command,
    check = "privilege_check",
    help_text_fn = "create_server_help",
)]
pub async fn create_server(
    ctx: Context<'_>,
    #[description = "Server identifier"]
    name: String,
    #[description = "Server address"]
    addr: String,
    #[description = "Maximum player count"]
    max_player_count: Option<u8>,
    #[description = "Is this a legacy CS:GO server"]
    legacy: Option<bool>,
    #[description = "Does the server require sv_allowupload 1"]
    allow_upload_required: Option<bool>,
) -> Result<(), Error> {
    let max_player_count: i64 = match max_player_count {
	Some(v) => v as i64,
	None => 16,
    };

    let legacy: bool = match legacy {
	Some(v) => v,
	None => true,
    };

    let allow_upload_required: bool = match allow_upload_required {
	Some(v) => v,
	None => false,
    };

    let server = Server {
	name: name.clone(),
	addr,
	max_player_count,
	legacy,
	allow_upload_required
    };

    let mut data = ctx.serenity_context().data.write().await;

    let conn = data.get_mut::<DbConnection>().ok_or("DataError: Unable to database connection")?;
    _ = db::write_server(&server, conn).await?;

    let servers = data.get_mut::<Servers>().ok_or("DataError: Unable to get servers")?;
    servers.insert(name, server);

    ctx.send(
	CreateReply::default()
	    .content("Successfully added server")
	    .ephemeral(true)
    ).await?;
    
    Ok(())
}

#[poise::command(slash_command)]
pub async fn delete_server(
    ctx: Context<'_>,
    #[description = "Server identifier"]
    name: String,
) -> Result<(), Error> {
    let mut data = ctx.serenity_context().data.write().await;

    let servers = data
        .get_mut::<Servers>()
        .ok_or("DataError: Unable to get servers")?;
    servers.retain(|s_name, _addr| *s_name != name);

    let sockets = data
        .get_mut::<ServerSocket>()
        .ok_or("DataError: Unable to get server sockets")?;
    sockets.remove(&name);

    let conn = data
        .get_mut::<DbConnection>()
        .ok_or("DataError: Unable to get database connection")?;
    remove_server(&name, conn).await?;

    ctx.send(
        CreateReply::default()
            .content(format!("Successfully deleted {}", name))
            .ephemeral(true),
    )
    .await?;
    
    Ok(())
}

#[poise::command(slash_command)]
pub async fn list_servers(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.serenity_context().data.read().await;

    let servers= data
        .get::<Servers>()
        .ok_or("DataError: Unable to get addresses")?;

    let list = servers
        .iter()
        .map(|(name, server)| format!("{} - {}\n", name, server.addr))
        .collect::<String>();

    ctx.send(
        CreateReply::default()
            .content(format!(
                r#"
```
{}
```
"#,
                list
            ))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

pub mod db {
    use super::*;
    use sqlx::SqliteConnection;

    /// Read Vec<Server> from the database `conn` and store each server by `name` in in the servers hashmap
    pub async fn read_servers(
	servers: &mut ServersValue,
	conn: &mut SqliteConnection,
    ) -> Result<(), Error> {
	let db_servers = sqlx::query_as!(Server, "SELECT * FROM server_settings")
	    .fetch_all(conn).await?;

	db_servers.into_iter().for_each(|server| {
	    servers.insert(server.name.clone(), server);
	});

	Ok(())
    }

    pub async fn write_server(
	server: &Server,
	conn: &mut SqliteConnection,
    ) -> Result<(), Error> {
	sqlx::query!(
	    "INSERT INTO server_settings (name, addr, max_player_count, legacy, allow_upload_required) VALUES (?, ?, ?, ?, ?)
ON CONFLICT(name) DO UPDATE
SET addr = excluded.addr,
    max_player_count = excluded.max_player_count,
    legacy = excluded.legacy,
    allow_upload_required = excluded.allow_upload_required",
	    server.name,
	    server.addr,
	    server.max_player_count,
	    server.legacy,
	    server.allow_upload_required
	    
	)
	    .execute(conn)
	    .await?;
	
	Ok(())
    }

    pub async fn remove_server(
	name: &String,
	conn: &mut SqliteConnection,
    ) -> Result<(), Error> {
	sqlx::query!("DELETE FROM server_settings WHERE name = ?", name)
	    .execute(conn)
	    .await?;

	Ok(())
    }

}
