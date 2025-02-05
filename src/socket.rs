use crate::serenity::standard::CommandResult;
use csgo_server::info::get_server_info;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::prelude::TypeMapKey;
use tokio::net::UdpSocket;

type Error = crate::Error;
type Context<'a> = crate::Context<'a>;

use csgo_server::request::create_socket;

pub struct ServerSocket;
impl TypeMapKey for ServerSocket {
    type Value = Option<UdpSocket>;
}

#[poise::command(slash_command)]
pub async fn set_address(
    ctx: Context<'_>,
    #[description = "Server address"] addr: core::net::SocketAddr,
) -> CommandResult {
    let sock = create_socket(addr).await?;

    let mut data = ctx.serenity_context().data.write().await;
    data.insert::<ServerSocket>(Some(sock));

    ctx.say(format!("Server address successfully updated to {}", addr)).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command, required_permissions = "SEND_MESSAGES")]
pub async fn status(ctx: Context<'_>) -> CommandResult {
    let mut data = ctx.serenity_context().data.read().await;
    let _ = match data.get::<ServerSocket>() {
	Some(Some(sock)) => {
	    let info = get_server_info(sock).await?;
	    ctx.say(format!("```{:#?}```", info)).await?
	}
	_ => ctx.say("No server address set").await?,
    };

    Ok(())
}
