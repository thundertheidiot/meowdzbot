use crate::server_info::Info;
use crate::ServerSocket;

use crate::server_info::get_server_info;
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use tokio::time;
use tokio::time::Duration;

use serenity::all::ActivityType;

pub async fn bot_status_loop(ctx: Arc<serenity::Context>) {
    let mut interval = time::interval(Duration::from_secs(2));

    loop {
        interval.tick().await;

        let data = ctx.data.read().await;
        match data.get::<ServerSocket>() {
            Some(socks) => match get_server_info(socks, &"meow".to_string()).await {
                Ok(info) => {
                    let status = bot_status(info);

                    ctx.set_activity(Some(serenity::ActivityData {
                        name: status,
                        kind: ActivityType::Playing,
                        state: None,
                        url: None,
                    }));
                }
                Err(e) => eprintln!("Error getting server information: {e}"),
            },
            None => eprintln!("DataError: Unable to get sockets"),
        }
    }
}

fn map_str(map: &str) -> String {
    let mut map = map;

    if map.chars().nth(2) == Some('_') {
        map = &map[3..];
    }

    String::from(map)
}

pub fn bot_status(info: Info) -> String {
    format!(
        "{} - {} players",
        map_str(&info.server_info.map),
        info.players.real().0.len()
    )
}
