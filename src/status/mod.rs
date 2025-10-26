pub mod activity;
mod slurs;
pub mod updating;

use crate::server_info::Info;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use crate::serenity::CreateActionRow;
use crate::server_info::get_server_info;
use crate::server_info::ServerDown;
use crate::server_info::ServerUp;
use crate::servers::Server;
use crate::servers::Servers;
use crate::settings::Settings;
use crate::socket::{ServerSocket, ServerSocketValue};
use crate::{Context, Error};
use ::serenity::all::{Colour, CreateAttachment, CreateEmbed, CreateEmbedFooter};
use csgo_server::players::Player;
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use urlencoding::encode;

use serenity::all::CreateButton;

pub async fn make_status_message(
    external_redirector: Option<String>,
    socks: &ServerSocketValue,
    name: &String, // not really required, but servers are stored as a hashmap so this will always be there anyway
    server: &Server,
) -> Result<(CreateEmbed, Vec<CreateActionRow>, Vec<CreateAttachment>), Error> {
    let info = get_server_info(socks, name).await?;

    let mut buttons: Vec<CreateButton> = vec![];
    let mut attachments = vec![CreateAttachment::path("static/respawnwcat.png").await?];
    let mut embed = CreateEmbed::new();

    match info {
        Info::ServerUp(info) => {
            let s_info = info.server_info;
            let players = info.players.real().0;

            if let Some(r) = external_redirector {
                buttons.push(
                    CreateButton::new_link(format!("{}/{}", r, encode(&server.addr)))
                        .label("Connect")
                        .emoji('📡'),
                );
            }

            if let Some(image) = info.image {
                attachments.push(CreateAttachment::path(format!("static/maps/{}", image)).await?);

                embed = embed.image(format!("attachment://{}", image));
            }

            embed = embed.title(s_info.name);

            // let max: usize = server.max_player_count as usize; // should be safe
            // match players.len() {
            //     0 => (),
            //     n if n < max => embed = embed.color(Colour::DARK_GREEN),
            //     n if n >= max => embed = embed.color(Colour::PURPLE),
            //     _ => (),
            // }

	    match players.len() {
                n if n > 5 && n <= 17 => embed = embed.color(Colour::DARK_GREEN),
                n if n > 17 => embed = embed.color(Colour::PURPLE),
                _ => (),
            }

            let connect_prefix = match server.allow_upload_required {
                true => "sv_allowupload 1; ",
                false => "",
            };

            embed = embed.description(format!(
                r#"
`{} - {} players online`
Time since map change `{:0>2}:{:0>2}`

Players
```
{}
```
Connect
```{}connect {}```
{}
If there are 16 or more players, you **must** connect through the console.
"#,
                s_info.map,
                players.len(),
                // max,
                (info.elapsed.as_secs() / 60) % 60,
                info.elapsed.as_secs() % 60,
                // discord breaks formatting of codeblocks if it's empty
                if players.len() > 0 {
                    format_players(players, &info.elapsed)
                } else {
                    " ".to_string()
                },
                connect_prefix,
                server.addr,
                if let (Some(stv), Some(pos)) = (s_info.source_tv, &server.addr.find(':')) {
                    format!(
                        "Spectate ```{}connect {}:{}```",
                        connect_prefix,
                        &server.addr[..*pos],
                        stv.port
                    )
                } else {
                    "".into()
                }
            ));

            embed = embed.thumbnail("attachment://respawnwcat.png");

            if !buttons.is_empty() {
                embed = embed.footer(CreateEmbedFooter::new(
                    "Open CS:GO before pressing connect!",
                ));
            }
        }
        Info::ServerDown(down) => {
            embed = embed.title("Server down");
            embed = embed.color(Colour::DARK_RED);

	    embed = embed.thumbnail("attachment://respawnwcat.png");
            embed = embed.description(format!(
                r#"
Server `{}` has been down since <t:{}:R>
"#,
                name,
		down.since.duration_since(UNIX_EPOCH)?.as_secs()
            ))
        }
    }

    let mut actions = vec![];
    if !buttons.is_empty() {
	actions.push(CreateActionRow::Buttons(buttons));
    }

    Ok((embed, actions, attachments))
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

    let server = data
        .get::<Servers>()
        .ok_or("DataError: Unable to get servers")?
        .get(&name)
        .ok_or(format!("ServerError: Unable to get server {}", &name))?;

    let (embed, action, attachments) =
        make_status_message(Some(redirect), socks, &name, server).await?;

    let mut message = CreateReply::default()
        .embed(embed)
        .components(action)
        .ephemeral(true);

    for a in attachments.into_iter() {
        message = message.attachment(a);
    }

    ctx.send(message).await?;

    Ok(())
}

// TODO per server warmup time
const WARMUP: f32 = 100.0;
fn playing_game(player: &Player, duration: &Duration) -> bool {
    player.duration > (duration.as_secs_f32() - WARMUP) || player.score > 0
}

use slurs::filter;
fn format_players(players: Vec<Player>, elapsed: &Duration) -> String {
    let (playing, not_playing): (Vec<Player>, Vec<Player>) =
        players.into_iter().partition(|p| playing_game(p, elapsed));

    if not_playing.len() > 0 {
        format!(
            r#"
{}
```
Waiting for next match (estimated)
```
{}
"#,
            if playing.len() > 0 {
                playing
                    .into_iter()
                    .map(|p| filter(&p.name))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                " ".into()
            },
            not_playing
                .into_iter()
                .map(|p| filter(&p.name))
                .collect::<Vec<_>>()
                .join("\n")
        )
    } else {
        playing
            .into_iter()
            .map(|p| filter(&p.name))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
