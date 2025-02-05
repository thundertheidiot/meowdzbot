use crate::server_info::status;
use crate::socket::ServerSocket;
use ::serenity::all::ActivityType;
use ::serenity::all::OnlineStatus;
use csgo_server::info::get_server_info;
use csgo_server::players::get_players;
use csgo_server::request::create_socket;
use db::read_server_address;
use db::DbConnection;
use db::ServerAddress;
use poise::serenity_prelude as serenity;
use server_info::bot_status;
use socket::set_address;
use socket::update_socket;
use sqlx::Connection;
use sqlx::SqliteConnection;
use std::collections::HashMap;
use std::sync::Arc;
use steam_redirect::steam_redirector_server;
use tokio::net::UdpSocket;
use tokio::time;

use std::env;
use std::io;
use std::time::Duration;

mod db;
mod server_info;
mod socket;
mod steam_redirect;

struct UserData {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, UserData, Error>;

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

async fn loop_timer(ctx: Arc<serenity::Context>) {
    let mut interval = time::interval(Duration::from_secs(10));

    loop {
        interval.tick().await;
        println!("loop");

        let data = ctx.data.read().await;
        let _ = match data.get::<ServerSocket>() {
            Some(v) => match v.get(&"meow".to_string()) {
                Some(sock) => {
                    if let Ok(status) = bot_status(sock).await {
                        ctx.set_activity(Some(serenity::ActivityData {
                            name: status,
                            kind: ActivityType::Playing,
                            state: None,
                            url: None,
                        }));
                    }
                }
                _ => continue,
            },
            // Some(Some(sock)) => {
            // }
            _ => continue,
        };
    }
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, UserData, Error>,
    data: &UserData,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);

            tokio::spawn(loop_timer(Arc::new(ctx.clone())));
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("DISCORD_TOKEN").expect("Set $DISCORD_TOKEN to your discord token.");
    let db_address =
        env::var("DATABASE_URL").expect("Set $DATABASE_URL to your an sqlite database.");
    // let addr =
    //     env::var("SERVER_ADDRESS").expect("Set $SERVER_ADDRESS to the address of the server.");

    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            commands: vec![register(), status(), set_address()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(UserData {})
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    {
        let mut data = client.data.write().await;
        let mut conn = SqliteConnection::connect(&db_address).await?;

        sqlx::migrate!().run(&mut conn).await?;

	let addresses = read_server_address(&mut conn).await?;

	let mut sockets = HashMap::new();
	for (n, a) in addresses.clone() {
	    update_socket(&mut sockets, n, a).await?;
	}

	data.insert::<ServerSocket>(sockets);
        data.insert::<ServerAddress>(addresses);
        data.insert::<DbConnection>(conn);
    }

    tokio::spawn(steam_redirector_server());

    client.start().await?;

    Ok(())
}
