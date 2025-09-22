use std::collections::HashMap;
use axum::extract::Path;
use axum::{routing::get, Json, Router};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use riven::{RiotApi, RiotApiConfig};
use riven::consts::{PlatformRoute, RegionalRoute};
use riven::models::account_v1::Account;
use riven::models::league_v4::LeagueEntry;
use riven::models::match_v5::Match;

struct AppCache {
    matches_cache: Arc<Mutex<HashMap<String, Match>>>, // match_id: MatchDto
    account_cache: Arc<Mutex<HashMap<String, Account>>>, // name#tag: AccountDto
    rank_cache: Arc<Mutex<HashMap<String, (LeagueEntry, Instant)>>>, // puuid: (LeagueEntryDto, last_updated)
}

struct AppState {
    riot_api: RiotApi,
    app_cache: AppCache,
    proxy_region: RegionalRoute,
}

impl AppCache {
    fn new() -> AppCache {
        AppCache {
            matches_cache: Arc::new(Mutex::new(HashMap::new())),
            account_cache: Arc::new(Mutex::new(HashMap::new())),
            rank_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

fn string_to_platform(platform: &str) -> Option<PlatformRoute> {
    match platform.to_lowercase().as_str() {
        "br1" => Some(PlatformRoute::BR1),
        "eun1" => Some(PlatformRoute::EUN1),
        "euw1" => Some(PlatformRoute::EUW1),
        "jp1" => Some(PlatformRoute::JP1),
        "kr" => Some(PlatformRoute::KR),
        "la1" => Some(PlatformRoute::LA1),
        "la2" => Some(PlatformRoute::LA2),
        "na1" => Some(PlatformRoute::NA1),
        "oc1" => Some(PlatformRoute::OC1),
        "tr1" => Some(PlatformRoute::TR1),
        "ru" => Some(PlatformRoute::RU),
        _ => None,
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let server_addr = env::var("SERVER_ADDR").unwrap_or("localhost".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or("7331".to_string());
    let riot_api_key = env::var("RIOT_API_KEY").expect("Riot API key not in environment variables.");
    let proxy_region = env::var("PROXY_REGION").unwrap_or("europe".to_string());

    let proxy_region = match proxy_region.as_str() {
        "europe" => RegionalRoute::EUROPE,
        "americas" => RegionalRoute::AMERICAS,
        "asia" => RegionalRoute::ASIA,
        "sea" => RegionalRoute::SEA,
        _ => panic!("proxy region is invalid, should be one of: europe, americas, asia, sea")
    };


    // Config documentation
    // https://github.com/MingweiSamuel/Riven/blob/v/2.x.x/riven/src/config.rs
    let riot_api = RiotApi::new(
        RiotApiConfig::with_key(riot_api_key)
            .set_retries(0) // no retries
            .set_burst_factor(0.9) // 90% burst factor
    );

    let shared_state = Arc::new(AppState {
        riot_api: riot_api,
        app_cache: AppCache::new(),
        proxy_region: proxy_region,
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
async fn handle_get_rank(
    Path((region, game_name, tag_line)): Path<(String, String, String)>,
    state: Arc<AppState>,
) -> Result<Json<Vec<LeagueEntry>>, (axum::http::StatusCode, Json<Value>)> {
    let account = state
        .riot_api
        .account_v1()
        .get_by_riot_id(state.proxy_region, &game_name, &tag_line)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )
        })?;

    let account = match account {
        Some(a) => a,
        None => {
            return Err((
                axum::http::StatusCode::NOT_FOUND,
                Json(json!({ "error": "Account not found" })),
            ));
        }
    };

    if let Some(platform) = string_to_platform(&region) {
        let rank = state
            .riot_api
            .league_v4()
            .get_league_entries_by_puuid(platform, &account.puuid)
            .await
            .map_err(|e| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": e.to_string() })),
                )
            })?;

        Ok(Json(rank))
    } else {
        Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid region" })),
        ))
    }
}
