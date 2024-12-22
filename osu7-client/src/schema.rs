use getset::CopyGetters;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Data {
    #[serde(rename = "ppIfMapEndsNow")]
    pp_ends_now: f64,
    #[serde(rename = "ppIfRestFced")]
    pp_if_fc: f64,
    #[serde(rename = "acc")]
    accuracy: f64,
    #[serde(rename = "unstableRate")]
    unstable_rate: f64,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Gameplay {
    pp: PerformancePoints,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct PerformancePoints {
    current: f64,
}
