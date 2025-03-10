use crate::servers::create_server;
use crate::servers::delete_server;
use crate::servers::Servers;
use crate::socket::ServerSocket;
use crate::status::activity::bot_status_loop;
use crate::status::status;
use crate::webserver::server;
use db::DbConnection;
use down_detector::down_detector_loop;
use once_cell::sync::Lazy;
use poise::samples::on_error;
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use servers::db::read_servers;
use servers::list_servers;
use servers::Server;
use settings::db::read_settings;
use settings::set_external_redirector;
use socket::update_socket;
use socket::ServerSocketValue;
use sqlx::Connection;
use sqlx::SqliteConnection;
use status::updating::create_updating_status;
use status::updating::db::read_updating_status_messages;
use status::updating::delete_updating_status;
use status::updating::status_message_update_loop;
use status::updating::UpdatingStatusMessages;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use std::env;

mod db;
mod down_detector;
mod gamestate_integration;
mod server_info;
mod servers;
mod settings;
mod socket;
mod status;
mod webserver;
// mod queue;

struct UserData {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, UserData, Error>;

#[poise::command(prefix_command, hide_in_help = true)]
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

#[poise::command(slash_command, check = "privilege_check")]
async fn restart(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(
        CreateReply::default()
            .content("restarting...")
            .ephemeral(true),
    )
    .await?;

    std::process::exit(0);
}

async fn error_handler<U, E: Display + Debug>(error: poise::FrameworkError<'_, U, E>) {
    if let poise::FrameworkError::Command { error, ctx, .. } = error {
        if let Err(e) = ctx
            .send(
                CreateReply::default()
                    .content(format!("{error}"))
                    .ephemeral(true),
            )
            .await
        {
            eprintln!("unable to send error message: {e}");
        }
    } else {
        _ = on_error(error).await;
    }
}

static TASKS: Lazy<Arc<RwLock<Vec<JoinHandle<()>>>>> =
    Lazy::new(|| Arc::new(RwLock::new(Vec::new())));
async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, UserData, Error>,
    _data: &UserData,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);

            let tasks = vec![
                tokio::spawn(bot_status_loop(Arc::new(ctx.clone()))),
                tokio::spawn(status_message_update_loop(Arc::new(ctx.clone()))),
                tokio::spawn(down_detector_loop(Arc::new(ctx.clone()))),
            ];

            let mut t = TASKS.write().await;
            t.clear();
            t.extend(tasks);
        }
        serenity::FullEvent::Resume { event, .. } => {
            println!("resumed, {event:#?}");

            let mut t = TASKS.write().await;
            for task in t.iter() {
                task.abort();
            }

            let tasks = vec![
                tokio::spawn(bot_status_loop(Arc::new(ctx.clone()))),
                tokio::spawn(status_message_update_loop(Arc::new(ctx.clone()))),
                tokio::spawn(down_detector_loop(Arc::new(ctx.clone()))),
            ];

            t.clear();
            t.extend(tasks);
        }
        _ => {}
    }
    Ok(())
}

async fn privilege_check(ctx: Context<'_>) -> Result<bool, Error> {
    let id = ctx.author().id.get();

    let thunder = id == 349607458324348930;
    let yugge = id == 224271935989612545;
    let strawbarry = id == 270112207344369665;

    Ok(thunder || yugge || strawbarry)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("DISCORD_TOKEN").expect("Set $DISCORD_TOKEN to your discord token.");
    let db_address = env::var("DATABASE_URL").expect("Set $DATABASE_URL to your sqlite database.");

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
            on_error: |err| Box::pin(error_handler(err)),
            commands: vec![
                register(),
                help(),
                restart(),
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

                // Webserver
                tokio::spawn(server(Arc::new(ctx.clone())));

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

        let mut servers: HashMap<String, Server> = HashMap::new();
        read_servers(&mut servers, &mut conn).await?;

        let mut sockets: ServerSocketValue = HashMap::new();
        for (n, a) in &servers {
            match update_socket(&mut sockets, n.clone(), &a.addr).await {
                Ok(_) => (),
                Err(e) => eprintln!("Unable to create socket: {e:#?}."),
            };
        }

        read_settings(&mut data, &mut conn).await?;
        data.insert::<UpdatingStatusMessages>(read_updating_status_messages(&mut conn).await?);
        data.insert::<ServerSocket>(sockets);
        data.insert::<Servers>(servers);
        data.insert::<DbConnection>(conn);
    }

    client.start().await?;

    Ok(())
}
