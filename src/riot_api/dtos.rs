use serde::{Deserialize, Serialize};

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