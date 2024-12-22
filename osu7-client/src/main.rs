use app::App;
use core::Core;
use std::sync::mpsc;

mod app;
mod core;
mod schema;

#[derive(Debug, Clone, Copy)]
enum ChannelMsg {
    ChangeDisplayStat(Statistic),
    ChangeDisplayBrightness(Brightness),
    DisplayConnected(bool),
    WebsocketConnected(bool),
    AppExit,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Statistic {
    PerformanceIfFC,
    PerformanceIfEndsNow,
    Accuracy,
    UnstableRate,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Brightness {
    Minimum,
    Medium,
    Maximum,
}

const DEFAULT_URL: &str = "ws://localhost:24050/tokens?bulkUpdates=MainPipeline,LiveToken";

fn main() {
    let url = std::fs::read_to_string("./url.txt").unwrap_or(DEFAULT_URL.to_owned());

    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    let handle = Core::run(rx1, tx2, url);

    App::run(tx1, rx2);

    handle.join().expect("Thread crashed");
}
