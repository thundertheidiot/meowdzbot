use crate::servers::Servers;
use crate::Error;
use crate::ServerSocket;
use crate::settings::Settings;

use crate::server_info::get_server_info;
use poise::serenity_prelude as serenity;
use ::serenity::prelude::TypeMap;
use tokio::sync::RwLockReadGuard;
use std::sync::Arc;
use tokio::time;
use tokio::time::Duration;

use serenity::all::ActivityType;

async fn bot_status(data: RwLockReadGuard<'_, TypeMap>) -> Result<String, Error> {
    let settings = data.get::<Settings>().ok_or("DataError: Unable to get settings")?;
	let ident = match settings.activity_server_identifier.as_ref() {
	    Some(v) => v,
	    None => &"meow".to_string(),
	};

    let socks = data.get::<ServerSocket>().ok_or("DataError: Unable to get sockets")?;
    let servers = data.get::<Servers>().ok_or("DataError: Unable to get servers")?;

    let info = get_server_info(socks, ident).await?;
    let server = servers.get(ident).ok_or(format!("ServerError: Unable to get server {}", ident))?;
    
    Ok(format!(
        "{} - {}/{} players",
        map_str(&info.server_info.map),
        info.players.real().0.len(),
	server.max_player_count
    ))
}

fn map_str(map: &str) -> String {
    let mut map = map;

    if map.chars().nth(2) == Some('_') {
        map = &map[3..];
    }

    String::from(map)
}

pub async fn bot_status_loop(ctx: Arc<serenity::Context>) {
    let mut interval = time::interval(Duration::from_secs(2));

    loop {
        interval.tick().await;
        let data = ctx.data.read().await;

	match bot_status(data).await {
	    Ok(status) => ctx.set_activity(Some(serenity::ActivityData {
		name: status,
		kind: ActivityType::Playing,
		state: None,
		url: None,
	    })),
	    Err(e) => eprintln!("{e}"),
	}
    }
}
