use reqwest::Client;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::{Instant}};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct LeagueEntryDto {
    pub queueType: String,
    pub tier: String,
    pub rank: String,
    pub leaguePoints: i32,
    pub wins: i32,
    pub losses: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct AccountDto {
    pub puuid: String,
    pub gameName: String,
    pub tagLine: String,
}

#[derive(Debug)]
pub enum RiotApiError {
    Network(reqwest::Error),
    Serde(serde_json::Error),
    Status(reqwest::StatusCode),
    Other(String),
}

impl std::fmt::Display for RiotApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiotApiError::Network(e) => write!(f, "network error: {}", e),
            RiotApiError::Serde(e) => write!(f, "json parse error: {}", e),
            RiotApiError::Status(code) => write!(f, "riot api returned non-success status: {}", code),
            RiotApiError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for RiotApiError {}

impl From<reqwest::Error> for RiotApiError {
    fn from(err: reqwest::Error) -> Self {
        RiotApiError::Network(err)
    }
}

impl From<serde_json::Error> for RiotApiError {
    fn from(err: serde_json::Error) -> Self {
        RiotApiError::Serde(err)
    }
}

pub type Result<T> = std::result::Result<T, RiotApiError>;

struct CacheEntry {
    value: Value,
    inserted: Instant,
    ttl: Option<Duration> // None = infinite duration
}

pub struct RiotApi {
    client: Client,
    api_key: String,
    proxy_region: String,
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl RiotApi {
    pub fn new(api_key: String, proxy_region: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            proxy_region,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn get<T>(&self, url: &str, check_cache: bool, ttl: Duration) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Clone
    {
        let mut cache = self.cache.lock().await;

        if check_cache {
            if let Some(entry) = cache.get(url) {
                if let Some(ttl) = entry.ttl {
                    if entry.inserted.elapsed() < ttl {
                        if let Ok(deserialized) = serde_json::from_value::<T>(entry.value.clone()) {
                            return Ok(deserialized);
                        }
                    }
                } else {
                    cache.remove(url);
                }
            }
        }

        let res = self.client
            .get(url)
            .header("X-Riot-Token", &self.api_key)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(RiotApiError::Status(res.status()));
        }

        let json: T = res.json().await?;
        cache.insert(url.to_string(), CacheEntry {
            value: serde_json::to_value(&json)?,
            inserted: Instant::now(),
            ttl: Option::from(ttl),
        });

        Ok(json)
    }

    pub async fn get_account(&self, game_name: &str, tag_line: &str) -> Result<AccountDto> {
        let url = format!(
            "https://{}.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}",
            self.proxy_region, game_name, tag_line
        );

        self.get(&url, true, Duration::from_secs(1000)).await
    }

    pub async fn get_league(&self, game_region: &str, puuid: &str) -> Result<Vec<LeagueEntryDto>> {
        let url = format!(
            "https://{}.api.riotgames.com/lol/league/v4/entries/by-puuid/{}",
            game_region, puuid
        );

        self.get(&url, true, Duration::from_secs(60)).await
    }

    pub async fn get_match(&self, game_region: &str, match_id: &str) -> Result<Value> {
        let url = format!(
            "https://{}.api.riotgames.com/lol/match/v5/matches/{}",
            game_region, match_id
        );
        self.get(&url, true, Duration::from_secs(600)).await
    }
}
