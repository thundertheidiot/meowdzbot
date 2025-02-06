use crate::Error;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use urlencoding::decode;

pub async fn steam_redirector_server() -> Result<(), Error> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Steam redirector listening on 0.0.0.0:8080");

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buffer = [0; 1024];

            match socket.read(&mut buffer).await {
                Ok(v) => {
                    let request = String::from_utf8_lossy(&buffer);
                    let first_line: &str = match request.lines().next() {
                        Some(v) => v,
                        None => {
                            eprintln!("Failed to read request");
                            return;
                        }
                    };

                    let path = first_line.split_whitespace()
			.nth(1).unwrap_or("/");
		    let path = decode(path).unwrap_or("invalid path".into());

                    let response_body = format!(
                        r#"<html><head><script>window.open("steam://connect/{}", "_self");</script></head></html>"#,
			path
                    );

                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                        response_body.len(),
                        response_body
                    );

                    if let Err(e) = socket.write_all(response.as_bytes()).await {
                        eprintln!("Failed to write response: {e}")
                    }
                }
                Err(e) => eprintln!("Failed to read from socket: {e}"),
            }
        });
    }
}
