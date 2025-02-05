use tokio::sync::RwLockWriteGuard;
use std::collections::HashMap;

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

use std::sync::{Arc, RwLock};

pub async fn update_socket<T: ToSocketAddrs>(
    sockets: &mut HashMap<String, UdpSocket>,
    name: String,
    addr: T,
) -> Result<(), Error> {
    let sock = create_socket(addr).await?;

    sockets.insert(name, sock);

    Ok(())
}

#[poise::command(slash_command)]
pub async fn set_address(
    ctx: Context<'_>,
    #[description = "Server identifier"] name: String,
    #[description = "Server address"] addr: String,
) -> CommandResult {
    let mut data = ctx.serenity_context().data.write().await;

    dbg!(&name);
    dbg!(&addr);

    let addrs = data
        .get_mut::<ServerAddress>()
        .ok_or("Unable to get addresses")?;
    _ = addrs.insert(name.clone(), addr.clone());

    dbg!(&addrs);

    ctx.say(format!("Server `{}` address set to {}", &name, &addr))
        .await?;

    let addrs = data
        .get::<ServerAddress>()
        .ok_or("Unable to get read only addresses")?
        .clone();
    let conn = data
        .get_mut::<DbConnection>()
        .ok_or("Unable to get database")?;
    store_server_address(conn, addrs).await?;

    let sockets = data.get_mut::<ServerSocket>().ok_or("Unable to get sockets")?;
    update_socket(sockets, name, addr).await?;

    Ok(())
}
