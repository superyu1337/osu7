use getset::CopyGetters;

pub mod streamcompanion;
pub mod tosu;

#[derive(Default, Debug, Clone, Copy, CopyGetters)]
#[get_copy = "pub"]
pub struct OsuData {
    pp_current: f64,
    pp_fc: f64,
    accuracy: f64,
    unstable_rate: f64,
}
