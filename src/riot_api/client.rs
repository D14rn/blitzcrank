use crate::riot_api::dtos::{AccountDto, LeagueEntryDto, MatchDto};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;

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

pub struct RiotApi {
    client: Client,
    api_key: String,
    proxy_region: String,
}

impl RiotApi {
    pub fn new(api_key: String, proxy_region: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            proxy_region,
        }
    }

    async fn get<T>(&self, url: &str,) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Clone
    {
        let res = self.client
            .get(url)
            .header("X-Riot-Token", &self.api_key)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(RiotApiError::Status(res.status()));
        }

        let json: T = res.json().await?;

        Ok(json)
    }

    pub async fn get_account(&self, game_name: &str, tag_line: &str) -> Result<AccountDto> {
        let url = format!(
            "https://{}.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}",
            self.proxy_region, game_name, tag_line
        );

        self.get(&url).await
    }

    pub async fn get_league(&self, game_region: &str, puuid: &str) -> Result<Vec<LeagueEntryDto>> {
        let url = format!(
            "https://{}.api.riotgames.com/lol/league/v4/entries/by-puuid/{}",
            game_region, puuid
        );

        self.get(&url).await
    }

    pub async fn get_match(&self, game_region: &str, match_id: &str) -> Result<MatchDto> {
        let url = format!(
            "https://{}.api.riotgames.com/lol/match/v5/matches/{}",
            game_region, match_id
        );
        self.get(&url).await
    }
}
