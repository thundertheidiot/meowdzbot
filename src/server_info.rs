use crate::db::{DbConnection, ServerAddress};
use crate::ServerSocket;
use ::serenity::all::{CreateActionRow, CreateButton};
use csgo_server::players::Player;
use csgo_server::{info::get_server_info, players::get_players};
use tokio::net::UdpSocket;
use urlencoding::encode;

use crate::Context;
use crate::Error;

use crate::serenity::standard::CommandResult;
use poise::{serenity_prelude as serenity, CreateReply};

async fn get_real_player_count(sock: &UdpSocket) -> Result<usize, Error> {
    Ok(get_players(sock).await?.real().0.len())
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

    Ok(format!("{} - {} players", map_str(&info.map), players))
}

fn format_players(mut players: Vec<Player>) -> String {
    players.sort_by(|a, b| b.score.cmp(&a.score));

    players
        .into_iter()
        .enumerate()
        .map(|(i, p)| format!("{: >2}. {:^20} --- score: {}\n", i + 1, p.name, p.score))
        .collect::<String>()
}

#[poise::command(slash_command, prefix_command, required_permissions = "SEND_MESSAGES")]
pub async fn status(
    ctx: Context<'_>,
    #[description = "Server identifier"] name: String,
) -> CommandResult {
    let data = ctx.serenity_context().data.read().await;

    let socks = data.get::<ServerSocket>().ok_or("Unable to get sockets")?;

    match socks.get(&name) {
	Some(sock) => {
            let info = get_server_info(sock).await?;
            let mut players = get_players(sock).await?.real().0;

	    let button = match data.get::<ServerAddress>().and_then(|v| v.get(&name)) {
		Some(v) => vec![
		    CreateButton::new_link(
			format!("http://localhost:8080/{}",
				encode(v))
		    ).label("Connect").emoji('🐈')
		],
		None => vec![],
	    };

            let content = format!(
                r#"
{}
`{} - {} players online`
Player list:
```
{}
```
Open CS:GO before pressing connect
"#,
                info.name,
                info.map,
                players.len(),
                format_players(players),
            );

	    // ctx.say(content).await?
	    ctx.send(
		CreateReply::default().content(content)
		    .components(vec![CreateActionRow::Buttons(button)])
	    ).await?
	},
	None => ctx.say("ligma").await?,
    };

    Ok(())
}
