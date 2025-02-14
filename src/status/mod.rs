pub mod activity;
pub mod updating;

use crate::db::ServerAddress;
use crate::serenity::CreateActionRow;
use crate::server_info::get_server_info;
use crate::settings::Settings;
use crate::socket::ServerSocket;
use crate::{Context, Error};
use ::serenity::all::{Colour, CreateEmbed, CreateEmbedFooter};
use csgo_server::players::Player;
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use std::collections::HashMap;
use tokio::net::UdpSocket;
use urlencoding::encode;

use serenity::all::CreateButton;
use serenity::standard::CommandResult;

pub async fn make_status_message(
    external_redirector: Option<String>,
    socks: &HashMap<String, UdpSocket>,
    name: &String,
    address: Option<&str>,
) -> Result<(CreateEmbed, Vec<CreateActionRow>), Error> {
    let info = get_server_info(socks, name).await?;
    let s_info = info.server_info;
    let players = info.players.real().0;

    let button = match (external_redirector, address) {
        (Some(r), Some(a)) => vec![CreateButton::new_link(format!("{}/{}", r, encode(a)))
            .label("Connect")
            .emoji('📡')],
        _ => vec![],
    };

    let mut embed = CreateEmbed::new()
        .title(s_info.name)
        .color(Colour::DARK_PURPLE) // TODO make color reflect player count
        .description(format!(
            r#"
`{} - {} players online`

```
{}
```
"#,
            s_info.map,
            players.len(),
            format_players(players)
        ));

    if !button.is_empty() {
        embed = embed.footer(CreateEmbedFooter::new(
            "Open CS:GO before pressing connect!",
        ));
    }

    Ok((embed, vec![CreateActionRow::Buttons(button)]))
}

#[poise::command(slash_command, required_permissions = "SEND_MESSAGES")]
pub async fn status(
    ctx: Context<'_>,
    #[description = "Server identifier"] name: String,
) -> CommandResult {
    let data = ctx.serenity_context().data.read().await;

    let settings = data
        .get::<Settings>()
        .ok_or("DataError: Unable to fetch settings")?;
    let redirect = settings
        .external_redirector_address
        .clone()
        .unwrap_or_default();
    let socks = data
        .get::<ServerSocket>()
        .ok_or("DataError: Unable to get sockets")?;

    let (embed, action) = make_status_message(
        Some(redirect),
        socks,
        &name,
        data.get::<ServerAddress>()
            .and_then(|v| v.get(&name).and_then(|v| Some(v.as_str()))),
    )
    .await?;

    ctx.send(CreateReply::default().embed(embed).components(action).ephemeral(true))
        .await?;

    Ok(())
}

fn format_players(players: Vec<Player>) -> String {
    // players.sort_by(|a, b| b.score.cmp(&a.score));

    // players
    //     .into_iter()
    //     .enumerate()
    //     .map(|(i, p)| format!("{: >2}. {:^20} --- score: {}\n", i + 1, p.name, p.score))
    //     .collect::<String>()

    players
        .into_iter()
        .map(|p| p.name + "\n")
        .collect::<String>()
}
