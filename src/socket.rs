use std::collections::HashMap;
use std::io;

use poise::serenity_prelude::prelude::TypeMapKey;
use tokio::net::{ToSocketAddrs, UdpSocket};

type Error = crate::Error;

pub struct ServerSocket;
// The bot had a weird issue, where it would mix up the info and player query data, this seems to fix that
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

pub async fn update_socket<A: ToSocketAddrs>(
    sockets: &mut ServerSocketValue,
    name: String,
    addr: A,
) -> Result<(), Error> {
    let socks = create_sockets(addr).await?;

    sockets.insert(name, socks);

    Ok(())
}
