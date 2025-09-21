use serde::{Deserialize, Serialize};

// Match-V5
#[derive(Clone, Serialize, Deserialize)]
pub struct MatchDto {
    pub metadata: MetadataDto,
    pub info: InfoDto,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataDto {
    pub data_version: String,
    pub match_id: String,
    pub participants: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoDto {
    pub end_of_game_result: String,
    pub game_start_timestamp: i64,
    pub game_end_timestamp: i64,
    pub game_creation: i64,
    pub game_duration: i64,
    pub game_id: i64,
    pub game_mode: String,
    pub game_name: String,
    pub game_type: String,
    pub game_version: String,
    pub map_id: i64,
    pub participants: Vec<ParticipantDto>,
    pub platform_id: String,
    pub queue_id: i64,
    pub teams: Vec<TeamDto>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamDto {
    team_id: i64,
    win: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantDto {
    pub kills: i64,
    pub assists: i64,
    pub deaths: i64,
    pub champion_id: i64,
    pub champion_name: String,
    pub participant_id: i64,
    pub puuid: String,
    pub riot_id_game_name: String,
    pub riot_id_tagline: String,
    pub win: bool,
}


