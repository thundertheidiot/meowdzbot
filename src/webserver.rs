use crate::server_info::Info;
use axum::extract::State;
use axum::Json;
use axum::{
    body::{to_bytes, HttpBody},
    extract::{Path, Request},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
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
    let html_content = format!(
        r#"<!DOCTYPE html><html><head><script>window.open("steam://connect/{}", "_self");window.close();</script></head></html>"#,
        decode(&path).unwrap_or("INVALID".into())
    );
    Html(html_content)
}

async fn handle_get() -> impl IntoResponse {
    let html_content = "hi";

    Html(html_content)
}

async fn server_data(
    State(ctx): State<Arc<serenity::Context>>,
    Path(path): Path<String>,
) -> Result<Json<Info>, impl IntoResponse> {
    let data = ctx.data.read().await;
    let socks = match data.get::<ServerSocket>() {
        Some(v) => v,
        None => {
            return Err(Json("DataError: Unable to fetch sockets".to_string()));
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

pub async fn server(ctx: Arc<serenity::Context>) -> Result<(), Error> {
    let app = Router::new()
        .route("/", post(gamestate_handler))
        .route("/", get(handle_get))
        .route("/data/{*path}", get(server_data))
        .route("/{*path}", get(steam_connect))
        .with_state(ctx);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on http://0.0.0.0:8080");

    axum::serve(listener, app).await?;

    Ok(())
}
