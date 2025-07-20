use crate::server_info::Info;
use axum::extract::State;
use axum::Json;
use axum::{
    body::to_bytes,
    extract::{Path, Request},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::from_slice;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use urlencoding::decode;

use poise::serenity_prelude as serenity;
use poise::serenity_prelude::prelude::TypeMapKey;

use crate::server_info::get_server_info;
use crate::socket::ServerSocket;
use crate::Error;

mod style;
use style::{ClassName, STYLE_SHEET};

#[derive(Deserialize, Debug)]
struct GameStateIntegrationData {
    token: String,
}

impl TypeMapKey for GameStateIntegrationData {
    // name, data
    type Value = HashMap<String, GameStateIntegrationData>;
}

struct GameStateIntegrationTokens;
impl TypeMapKey for GameStateIntegrationTokens {
    // token, name
    type Value = HashMap<String, String>;
}

async fn gamestate_handler(
    State(ctx): State<Arc<serenity::Context>>,
    request: Request,
) -> impl IntoResponse {
    let body_bytes = to_bytes(request.into_body(), usize::MAX)
        .await
        .unwrap_or_default();

    match from_slice::<GameStateIntegrationData>(&body_bytes) {
        Ok(payload) => {
            println!("Recieved JSON: {:?}", payload);

            tokio::spawn(async move {
                let mut data = ctx.data.write().await;

                let tokens = data.get::<GameStateIntegrationTokens>().unwrap();

                if let Some(name) = tokens.get(&payload.token) {
                    let name = name.clone();

                    match data.get_mut::<GameStateIntegrationData>() {
                        Some(gsi) => {
                            _ = gsi.insert(name, payload);
                        }
                        None => eprintln!("Unable to get gamestate_integration data."),
                    };
                }
            });

            (StatusCode::OK, "OK")
        }
        Err(e) => {
            println!("Invalid JSON: {:?}", e);
            (StatusCode::BAD_REQUEST, "Invalid JSON")
        }
    }
}

async fn steam_connect(Path(path): Path<String>) -> impl IntoResponse {
    Html(maud::html! {
	head {
	    style {(PreEscaped(STYLE_SHEET))}
	    link rel="icon" type="image/png" href="/static/favicon.png" {}

	    script {
		(PreEscaped(format!(r#"window.open("steam://connect/{}", "_self");"#,
				    decode(&path).unwrap_or("INVALID".into())
		)))
	    }
	}

	body {
	    video class=(ClassName::VID) autoplay loop muted playsinline {
		source src = "/static/dangerzone.mp4" type="video/mp4" {}
	    }

	    br style=(PreEscaped("padding-top: 20px;")) {}

	    div {
		p style="font-size: 150%;" {
		    "Connecting you to: "
			code { (PreEscaped(format!("{path}"))) }
		}

		p {
		    b { "Open CS:GO before connecting!" }
		    " Otherwise Steam will automatically open CS2."
		}

		p {
		    "Click \"Open Link\"/\"Open Steam\" on the popup to connect, to skip this in the future, check the always allow box."
			" If nothing opened, you may have to disable popup blocking."
		}

		br {}
		p style="font-size: smaller;" { "This tab can be closed after the steam window has appeared." }
	    }
	    
	}
    }.into_string())
}

use maud::PreEscaped;

const JAVASCRIPT: &'static str = include_str!("main.js");

async fn main_page() -> impl IntoResponse {
    Html(
	maud::html! {
	    head {
		style {(PreEscaped(STYLE_SHEET))}
		link rel="icon" type="image/png" href="/static/favicon.png" {}

		script { (PreEscaped(JAVASCRIPT)) }
	    }

	    body {
		video class=(ClassName::VID) autoplay loop muted playsinline {
		    source src = "/static/dangerzone.mp4" type="video/mp4" {}
		}

		br style=(PreEscaped("padding-top: 20px;")) {}

		div {
		    div id="meow" {
			p {"waiting for data"}
		    }

		    br {}

		    div id="meow2" {
			p {"waiting for data"}
		    }

		    br {}

		    p {
			"Join the " a href="https://discord.gg/hC82X4E2kF" { "Meow DZ Discord" } "for more information"
		    }
		}
		
	    }
	}
	.into_string())
}

/// Returns data for servers as a json blob, so that other people can integrate the bot data
/// Remember that player count does not represent the real player count, it has GOTV and spectators included
/// Filter out "DatHost - GOTV" and empty names
async fn server_data(
    State(ctx): State<Arc<serenity::Context>>,
    Path(path): Path<String>,
) -> Result<Json<Info>, impl IntoResponse> {
    let data = ctx.data.read().await;
    let socks = match data.get::<ServerSocket>() {
        Some(v) => v,
        None => {
            return Err(Json("DataError: Unable to get sockets".to_string()));
        }
    };

    let info = match get_server_info(socks, &path).await {
        Ok(v) => v,
        Err(e) => {
            return Err(Json(format!("Error: {e:?}")));
        }
    };

    Ok(Json(info))
}

use tower_http::services::ServeDir;

pub async fn server(ctx: Arc<serenity::Context>) -> Result<(), Error> {
    let app = Router::new()
        .route("/", post(gamestate_handler))
        .route("/", get(main_page))
        .route("/data/{*path}", get(server_data))
        .route("/{*path}", get(steam_connect))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(ctx);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on http://0.0.0.0:8080");

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Webserver error: {e}");
    };

    Ok(())
}
