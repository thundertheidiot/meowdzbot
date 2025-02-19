use poise::CreateReply;
use std::collections::HashMap;

use crate::db::remove_server_address;
use crate::db::{store_server_address, DbConnection, ServerAddress};
use poise::serenity_prelude::prelude::TypeMapKey;
use tokio::net::{ToSocketAddrs, UdpSocket};

type Error = crate::Error;
type Context<'a> = crate::Context<'a>;

pub struct ServerSocket;
pub type ServerSocketValue = HashMap<String, (UdpSocket, UdpSocket)>;
impl TypeMapKey for ServerSocket {
    type Value = ServerSocketValue;
}

async fn create_sockets<A: ToSocketAddrs>(address: A) -> io::Result<(UdpSocket, UdpSocket)> {
    let a = UdpSocket::bind("0.0.0.0:0").await?;
    let b = UdpSocket::bind("0.0.0.0:0").await?;

    a.connect(&address).await?;
    b.connect(&address).await?;

    Ok((a, b))
}

pub async fn update_socket<T: ToSocketAddrs>(
    sockets: &mut ServerSocketValue,
    name: String,
    addr: T,
) -> Result<(), Error> {
    let socks = create_sockets(addr).await?;

    sockets.insert(name, socks);

    Ok(())
}

fn create_server_help() -> String {
    "Adds a new server to the list, counterintuitively this also works for updating the address of an existing server.
Requires admin privileges."
        .into()
}

use crate::privilege_check;
#[poise::command(
    slash_command,
    check = "privilege_check",
    category = "Server",
    help_text_fn = "create_server_help"
)]
pub async fn create_server(
    ctx: Context<'_>,
    #[description = "Server identifier"] name: String,
    #[description = "Server address"] addr: String,
) -> Result<(), Error> {
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
) -> Result<(), Error> {
    let mut data = ctx.serenity_context().data.write().await;

    let addresses = data
        .get_mut::<ServerAddress>()
        .ok_or("DataError: Unable to get server addresses")?;
    addresses.retain(|s_name, _addr| *s_name != name);

    let sockets = data
        .get_mut::<ServerSocket>()
        .ok_or("DataError: Unable to get server sockets")?;
    sockets.remove(&name);

    let conn = data
        .get_mut::<DbConnection>()
        .ok_or("DataError: Unable to get database connection")?;
    remove_server_address(conn, &name).await?;

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

    let addrs = data
        .get::<ServerAddress>()
        .ok_or("DataError: Unable to get addresses")?;

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
