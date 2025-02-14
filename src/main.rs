use crate::socket::delete_server;
use crate::socket::ServerSocket;
use crate::status::activity::bot_status_loop;
use crate::status::status;
use crate::webserver::server;
use ::serenity::all::ActivityType;
use ::serenity::all::OnlineStatus;
use csgo_server::info::get_server_info;
use csgo_server::players::get_players;
use csgo_server::request::create_socket;
use db::read_server_address;
use db::read_settings;
use db::DbConnection;
use db::ServerAddress;
use poise::serenity_prelude as serenity;
use settings::set_external_redirector;
use socket::create_server;
use socket::list_servers;
use socket::update_socket;
use sqlx::Connection;
use sqlx::SqliteConnection;
use status::updating::create_updating_status;
use status::updating::db::read_updating_status_messages;
use status::updating::delete_updating_status;
use status::updating::status_message_update_loop;
use status::updating::UpdatingStatusMessages;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::time;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use std::env;
use std::io;
use std::time::Duration;

mod db;
mod gamestate_integration;
mod server_info;
mod settings;
mod socket;
mod status;
mod webserver;

struct UserData {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, UserData, Error>;

#[poise::command(
    prefix_command,
    hide_in_help = true
)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
    use poise::builtins::help;
    let config = poise::builtins::HelpConfiguration {
        ..Default::default()
    };

    help(ctx, command.as_deref(), config).await?;
    Ok(())
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, UserData, Error>,
    _data: &UserData,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);

            tokio::spawn(bot_status_loop(Arc::new(ctx.clone())));
            tokio::spawn(status_message_update_loop(Arc::new(ctx.clone())));
            tokio::spawn(server(Arc::new(ctx.clone())));
        }
        _ => {}
    }
    Ok(())
}

async fn privilege_check(ctx: Context<'_>) -> Result<bool, Error> {
    let thunder = ctx.author().id.get() == 349607458324348930;

    // Ok(thunder)
    Ok(false)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("DISCORD_TOKEN").expect("Set $DISCORD_TOKEN to your discord token.");
    let db_address =
        env::var("DATABASE_URL").expect("Set $DATABASE_URL to your sqlite database.");

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to setup tracing subscriber");

    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            commands: vec![
                register(),
                help(),
                status(),
                create_server(),
		delete_server(),
                list_servers(),
                set_external_redirector(),
                create_updating_status(),
                delete_updating_status(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
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
            match update_socket(&mut sockets, n, a).await {
                Ok(_) => (),
                Err(e) => eprintln!("Unable to create socket: {e:#?}."),
            };
        }

        read_settings(&mut data, &mut conn).await?;
        data.insert::<UpdatingStatusMessages>(read_updating_status_messages(&mut conn).await?);
        data.insert::<ServerSocket>(sockets);
        data.insert::<ServerAddress>(addresses);
        data.insert::<DbConnection>(conn);
    }

    client.start().await?;

    Ok(())
}
