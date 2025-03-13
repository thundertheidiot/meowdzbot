use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use crate::server_info::Info;
use crate::server_info::get_server_info;
use crate::server_info::INFO;
use crate::{Error, ServerSocket};
use ::serenity::all::CreateMessage;
use tokio::time;
use std::time::Duration;
use std::sync::Arc;
use poise::serenity_prelude as serenity;

// TODO unhardcode these
const CHID: u64 = 1224415507495649330;
const COP_CAT: u64 = 1223090099164549200;
// TODO store channel id + role/user id per game server

// TODO think about this more

pub async fn down_detector(ctx: &serenity::Context) -> Result<(), Error> {

    let data = ctx.data.read().await;

    let socks = data
        .get::<ServerSocket>()
        .ok_or("DataError: Unable to get sockets")?;

    for name in ["meow".to_string(), "meow2".to_string()] {
	if let Info::ServerDown(down) = get_server_info(socks, &name).await? {
	    if SystemTime::now().duration_since(down.since)?.as_secs() > 60 && !down.ping_sent {
		let channel = serenity::ChannelId::from(CHID);
		channel.send_message(&ctx, CreateMessage::new().content(
		    format!("<@&{}> Server `{name}` went down <t:{}:R>!",
			    COP_CAT,
			    down.since.duration_since(UNIX_EPOCH)?.as_secs()
		    )
		)).await?;

		let mut info = INFO.write().await;
		let info = info.get_mut(&name).ok_or(format!("MapDataError: Unable to get server {name}"))?;
		if let Info::ServerDown(down) = info {
		    down.ping_sent = true;
		} else {
		    println!("server is back up with incredible timing");
		}
	    }
	}
    }

    Ok(())
}

pub async fn down_detector_loop(ctx: Arc<serenity::Context>) {
    let mut interval = time::interval(Duration::from_secs(30));

    time::interval(Duration::from_secs(2)).tick().await;
    loop {
	interval.tick().await;

	match down_detector(&ctx).await {
	    Ok(_) => (),
	    Err(e) => eprintln!("Error with down detector {e:?}"),
	};
    }
}
