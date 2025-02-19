use tokio::time::Duration;
use tokio::net::ToSocketAddrs;
use tokio::net::UdpSocket;
use tokio::time;
use std::io;

pub async fn create_socket<A: ToSocketAddrs>(address: A) -> io::Result<UdpSocket> {
    let sock = UdpSocket::bind("0.0.0.0:0").await?;

    sock.connect(address).await?;

    Ok(sock)
}

#[derive(PartialEq)]
pub enum Query {
    /// A2S_INFO
    Info,
    /// A2S_PLAYER
    Player,
}

impl Query {
    pub fn get(&self) -> &'static [u8] {
	match self {
	    Query::Info => "TSource Engine Query\0".as_bytes(),
	    Query::Player => &[b'U', 0xFF, 0xFF, 0xFF, 0xFF],

	}
    }
}

pub async fn send_request(sock: &UdpSocket, query: Query) -> io::Result<Vec<u8>> {
    let mut buf = [0; 4096];
    let timeout = Duration::from_secs(5);

    let mut request: Vec<u8> = Vec::with_capacity(30);

    let header: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
    request.extend_from_slice(&header);

    request.extend_from_slice(query.get());

    _ = time::timeout(timeout, sock.send(&request)).await?;

    let len = time::timeout(timeout, sock.recv(&mut buf)).await?.unwrap();
    // Challenge mechanism
    while buf[4] == 0x41 {
	if query == Query::Player {
	    request.truncate(5); // FF FF FF FF 'U'
	}
	
	request.extend_from_slice(&buf[5..len]);
	_ = time::timeout(timeout, sock.send(&request)).await?;
	_ = time::timeout(timeout, sock.recv(&mut buf)).await?;
    }

    Ok(buf.to_vec())
}
