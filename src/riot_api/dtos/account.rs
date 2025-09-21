use serde::{Deserialize, Serialize};

// Account-V1
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDto {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
}