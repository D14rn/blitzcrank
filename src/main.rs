use std::env;
use std::sync::Arc;
use axum::{Router, routing::get, Json};
use axum::extract::Path;
use dotenv::dotenv;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use serde_json::Value;


struct AppState {
    client: Client,
    riot_api_key: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let riot_api_key = env::var("RIOT_API_KEY").expect("Riot API key not in environment variables.");
    let client = Client::new();

    let shared_state = Arc::new(AppState {
        client: client,
        riot_api_key: riot_api_key
    });

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/rank/{region}/{game_name}/{tag_line}",
            get({
                let shared = Arc::clone(&shared_state);
                move |path| handle_get_rank(path, shared)
            }),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7331").await.unwrap();
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

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct LeagueEntryDto {
    queueType: String,
    tier: String,
    rank: String,
    leaguePoints: i32,
    wins: i32,
    losses: i32,
}

async fn handle_get_rank(Path((region, game_name, tag_line)): Path<(String, String, String)>, state: Arc<AppState>) -> Json<Value> {
    match fetch_rank(&state.client, &region, &game_name, &tag_line, &state.riot_api_key).await {
        Ok(rank) => Json(rank),
        Err(e) => Json(serde_json::json!(
            {
                "error": e.to_string()
            }
        ))
    }
}

async fn fetch_rank(
    client: &Client,
    region: &str,
    game_name: &str,
    tag_line: &str,
    api_key: &str,
) -> Result<Value, Error> {
    let summoner_url = format!(
        "https://{}.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}?api_key={}",
        "europe", game_name, tag_line, api_key
    );

    let summoner_res: Value = client.get(&summoner_url).send().await?.json().await?;
    let summoner_id = summoner_res["puuid"].as_str().unwrap_or_default();

    // Step 2: Rank
    let rank_url = format!(
        "https://{}.api.riotgames.com/lol/league/v4/entries/by-puuid/{}?api_key={}",
        region, summoner_id, api_key
    );

    let rank_res: Value = client.get(&rank_url).send().await?.json().await?;

    Ok(rank_res)
}