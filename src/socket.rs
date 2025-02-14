use once_cell::sync::Lazy;
use poise::CreateReply;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::sync::RwLockWriteGuard;

use crate::db::{store_server_address, DbConnection, ServerAddress};
use crate::serenity::standard::CommandResult;
use csgo_server::info::get_server_info;
use poise::serenity_prelude::prelude::{TypeMap, TypeMapKey};
use tokio::net::{ToSocketAddrs, UdpSocket};

use poise::serenity_prelude as serenity;

type Error = crate::Error;
type Context<'a> = crate::Context<'a>;

use csgo_server::request::create_socket;

pub struct ServerSocket;
impl TypeMapKey for ServerSocket {
    type Value = HashMap<String, UdpSocket>;
}

use std::sync::Arc;

pub async fn update_socket<T: ToSocketAddrs>(
    sockets: &mut HashMap<String, UdpSocket>,
    name: String,
    addr: T,
) -> Result<(), Error> {
    let sock = create_socket(addr).await?;

    sockets.insert(name, sock);

    Ok(())
}

use crate::privilege_check;
#[poise::command(slash_command, check = "privilege_check")]
pub async fn create_server(
    ctx: Context<'_>,
    #[description = "Server identifier"] name: String,
    #[description = "Server address"] addr: String,
) -> CommandResult {
    let mut data = ctx.serenity_context().data.write().await;

    let addrs = data
        .get_mut::<ServerAddress>()
        .ok_or("Unable to get addresses")?;
    _ = addrs.insert(name.clone(), addr.clone());

    let addrs = addrs.clone();

    ctx.say(format!("Server `{}` address set to {}", &name, &addr))
        .await?;

    let conn = data
        .get_mut::<DbConnection>()
        .ok_or("Unable to get database")?;
    store_server_address(conn, addrs).await?;

    let sockets = data
        .get_mut::<ServerSocket>()
        .ok_or("Unable to get sockets")?;
    update_socket(sockets, name.clone(), addr).await?;

    Ok(())
}

#[poise::command(slash_command, check = "privilege_check")]
pub async fn delete_server(
    ctx: Context<'_>,
    #[description = "Server identifier"] name: String,
) -> CommandResult {
    let mut data = ctx.serenity_context().data.write().await;

    let addresses = data.get_mut::<ServerAddress>().ok_or("DataError: Unable to get server addresses")?;
    addresses.retain(|s_name, _addr| *s_name != name);

    let sockets = data.get_mut::<ServerSocket>().ok_or("DataError: Unable to get server sockets")?;
    sockets.remove(&name);


    let conn = data.get_mut::<DbConnection>().ok_or("DataError: Unable to get database connection")?;
    remove_server_address(conn, &name).await?;

    ctx.send(
	CreateReply::default()
	    .content(format!("Successfully deleted {}", name))
	    .ephemeral(true)
    ).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn list_servers(ctx: Context<'_>) -> CommandResult {
    let data = ctx.serenity_context().data.read().await;

    let addrs = data
        .get::<ServerAddress>()
        .ok_or("DataError: Unable to fetch addresses")?;

    let list = addrs
        .iter()
        .map(|(name, addr)| format!("{} - {}\n", name, addr))
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
