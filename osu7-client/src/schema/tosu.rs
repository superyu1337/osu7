use serde::{Deserialize, Serialize};

use super::OsuData;

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TosuResponse {
    gameplay: Gameplay,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Gameplay {
    pp: PerformancePoints,
    accuracy: f64,
    hits: Hits,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Hits {
    #[serde(rename = "unstableRate")]
    unstable_rate: f64,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PerformancePoints {
    current: f64,
    fc: f64,
}

impl TosuResponse {
    pub fn to_osu_data(self) -> OsuData {
        OsuData {
            pp_current: self.gameplay.pp.current,
            pp_fc: self.gameplay.pp.fc,
            accuracy: self.gameplay.accuracy,
            unstable_rate: self.gameplay.hits.unstable_rate,
        }
    }
}
