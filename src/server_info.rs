use csgo_server::{info::ServerInfo, players::Players};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

use csgo_server::info;

use crate::Error;

#[derive(Debug, Clone, Serialize)]
pub struct Info {
    pub server_info: ServerInfo,
    pub players: Players,
    timestamp: SystemTime,
}

static INFO: Lazy<Arc<RwLock<HashMap<String, Info>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub async fn get_server_info(
    socks: &HashMap<String, UdpSocket>,
    name: &String,
) -> Result<Info, Error> {
    let sock = socks
        .get(name)
        .ok_or(format!("SocketError: Unable to get socket of {}", name))?;

    let mut infomap = INFO.write().await;

    match infomap.get(name) {
        Some(v) => {
            if let Ok(duration) = SystemTime::now().duration_since(v.timestamp) {
                if duration.as_secs() >= 5 {
                    let server_info = info::get_server_info(sock).await?;
                    let players = csgo_server::players::get_players(sock).await?;

                    let info = Info {
                        server_info,
                        players,
                        timestamp: SystemTime::now(),
                    };

                    infomap.insert(name.clone(), info.clone());

                    return Ok(info);
                }
            }

            Ok(v.clone())
        }
        None => {
            println!("First data fetch for {}", name);

            let server_info = info::get_server_info(sock).await?;
            let players = csgo_server::players::get_players(sock).await?;

            let info = Info {
                server_info,
                players,
                timestamp: SystemTime::now(),
            };

            infomap.insert(name.clone(), info.clone());

            Ok(info)
        }
    }
}
