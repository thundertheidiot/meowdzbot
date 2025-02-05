use csgo_server::{info::get_server_info, players::get_players};
use tokio::net::UdpSocket;
use crate::Error;

async fn get_real_player_count(sock: &UdpSocket) -> Result<usize, Error> {
    Ok(
	get_players(sock).await?
	    .real().0.len()
    )
}

fn map_str(map: &str) -> String {
    let mut map = map;

    if map.chars().nth(2) == Some('_') {
	map = &map[3..];
    }

    String::from(map)
}

pub async fn bot_status(sock: &UdpSocket) -> Result<String, Error> {
    let info = get_server_info(sock).await?;
    let players = get_real_player_count(sock).await?;

    dbg!(&info);
    dbg!(&players);

    Ok(format!(
	"{} - {} players",
	map_str(&info.map),
	players
    ))
}
