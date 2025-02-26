use std::io;
use tokio::net::UdpSocket;
use tokio::time;
use tokio::time::Duration;

#[derive(PartialEq)]
pub enum Query {
    /// `A2S_INFO`
    Info,
    /// `A2S_PLAYER`
    Player,
}

impl Query {
    #[must_use]
    pub fn get(&self) -> &'static [u8] {
        match self {
            Query::Info => "TSource Engine Query\0".as_bytes(),
            Query::Player => &[b'U', 0xFF, 0xFF, 0xFF, 0xFF],
        }
    }
}

pub(crate) async fn send_request(sock: &UdpSocket, query: Query) -> io::Result<[u8; 4096]> {
    let mut buf = [0; 4096];
    let timeout = Duration::from_secs(5);

    let mut request: Vec<u8> = Vec::with_capacity(30);

    let header: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
    request.extend_from_slice(&header);

    request.extend_from_slice(query.get());

    _ = time::timeout(timeout, sock.send(&request)).await?;

    let len = time::timeout(timeout, sock.recv(&mut buf)).await??;
    // Challenge mechanism
    while buf[4] == 0x41 {
        if query == Query::Player {
            request.truncate(5); // FF FF FF FF 'U'
        }

        request.extend_from_slice(&buf[5..len]);
        _ = time::timeout(timeout, sock.send(&request)).await?;
        _ = time::timeout(timeout, sock.recv(&mut buf)).await?;
    }

    Ok(buf)
}
