use serde::{Deserialize, Serialize};

// League-V4
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueEntryDto {
    pub league_id: String,
    pub queue_type: String,
    pub tier: String,
    pub rank: String,
    pub puuid: String,
    pub league_points: i16,
    pub wins: u16,
    pub losses: u16,
    pub veteran: bool,
    pub inactive: bool,
    pub fresh_blood: bool,
    pub hot_streak: bool,
}
