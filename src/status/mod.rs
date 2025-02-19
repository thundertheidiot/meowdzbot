pub mod activity;
pub mod updating;

use crate::servers::Server;
use crate::serenity::CreateActionRow;
use crate::server_info::get_server_info;
use crate::servers::Servers;
use crate::settings::Settings;
use crate::socket::{ServerSocket, ServerSocketValue};
use crate::{Context, Error};
use ::serenity::all::{Colour, CreateEmbed, CreateEmbedFooter};
use csgo_server::players::Player;
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use urlencoding::encode;

use serenity::all::CreateButton;

pub async fn make_status_message(
    external_redirector: Option<String>,
    socks: &ServerSocketValue,
    name: &String, // not really required, but servers are stored as a hashmap
    server: &Server,
) -> Result<(CreateEmbed, Vec<CreateActionRow>), Error> {
    let info = get_server_info(socks, name).await?;
    let s_info = info.server_info;
    let players = info.players.real().0;

    let button = match external_redirector {
        Some(r) => vec![CreateButton::new_link(format!("{}/{}", r, encode(&server.addr)))
            .label("Connect")
            .emoji('📡')],
        _ => vec![],
    };

    let mut embed = CreateEmbed::new()
        .title(s_info.name)
        .color(Colour::DARK_PURPLE) // TODO make color reflect player count
        .description(format!(
            r#"
`{} - {}/{} players online`

```
{}
```
Connect manually:
`{}connect {}`
"#,
            s_info.map,
            players.len(),
	    server.max_player_count,
            format_players(players),
	    if server.allow_upload_required {"sv_allowupload 1; "} else {""},
	    server.addr
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
) -> Result<(), Error> {
    let data = ctx.serenity_context().data.read().await;

    let settings = data
        .get::<Settings>()
        .ok_or("DataError: Unable to get settings")?;
    let redirect = settings
        .external_redirector_address
        .clone()
        .unwrap_or_default();
    let socks = data
        .get::<ServerSocket>()
        .ok_or("DataError: Unable to get sockets")?;

    let server = data.get::<Servers>()
	.ok_or("DataError: Unable to get servers")?
	.get(&name).ok_or(format!("ServerError: Unable to get server {}", &name))?;

    let (embed, action) = make_status_message(
        Some(redirect),
        socks,
        &name,
	server,
    )
    .await?;

    ctx.send(
        CreateReply::default()
            .embed(embed)
            .components(action)
            .ephemeral(true),
    )
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
