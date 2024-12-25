use serde::{Deserialize, Serialize};

use super::OsuData;

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StreamCompanionResponse {
    #[serde(rename = "ppIfMapEndsNow")]
    pp_ends_now: Option<f64>,
    #[serde(rename = "ppIfRestFced")]
    pp_if_fc: Option<f64>,
    #[serde(rename = "acc")]
    accuracy: Option<f64>,
    #[serde(rename = "unstableRate")]
    unstable_rate: Option<f64>,
}

impl StreamCompanionResponse {
    pub fn to_osu_data(self, old_data: OsuData) -> OsuData {
        OsuData {
            pp_current: self.pp_ends_now.unwrap_or(old_data.pp_current),
            pp_fc: self.pp_if_fc.unwrap_or(old_data.pp_fc),
            accuracy: self.accuracy.unwrap_or(old_data.accuracy),
            unstable_rate: self.unstable_rate.unwrap_or(old_data.unstable_rate),
        }
    }
}