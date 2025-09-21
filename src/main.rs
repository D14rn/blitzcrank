mod riot_api;

use crate::riot_api::client::RiotApi;
use axum::extract::Path;
use axum::{routing::get, Json, Router};
use dotenv::dotenv;
use riot_api::dtos::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::sync::Arc;

struct AppState {
    riot_api: RiotApi,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let server_addr = env::var("SERVER_ADDR").unwrap_or("localhost".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or("7331".to_string());
    let riot_api_key = env::var("RIOT_API_KEY").expect("Riot API key not in environment variables.");
    let proxy_region = env::var("PROXY_REGION").unwrap_or("europe".to_string());

    let riot_api = RiotApi::new(riot_api_key, proxy_region);
    let shared_state = Arc::new(AppState {
        riot_api
    });

    let app = Router::new()
        .route(
            "/rank/{region}/{game_name}/{tag_line}",
            get({
                let shared = Arc::clone(&shared_state);
                move |path| handle_get_rank(path, shared)
            }),
        );

    let bind_addr = format!("{}:{}", server_addr, server_port);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Region {
    BR1,
    EUN1,
    EUW1,
    JP1,
    KR,
    LA1,
    LA2,
    ME1,
    NA1,
    OC1,
    RU,
    SG2,
    TR1,
    TW2,
    VN2
}

//#[debug_handler]
async fn handle_get_rank(Path((region, game_name, tag_line)): Path<(String, String, String)>, state: Arc<AppState>) -> Result<Json<Vec<LeagueEntryDto>>, (axum::http::StatusCode, Json<Value>)> {
    let account = state
        .riot_api
        .get_account(&game_name, &tag_line)
        .await
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))))?;

    let rank = state
        .riot_api
        .get_league(&region, &account.puuid)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(Json(rank))
}
