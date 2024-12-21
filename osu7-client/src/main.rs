use app::App;
use core::Core;
use std::sync::mpsc;

mod app;
mod core;
mod schema;

#[derive(Debug, Clone, Copy)]
enum ChannelMsg {
    ChangeDisplayStat(Statistic),
    DisplayConnected(bool),
    WebsocketConnected(bool),
    AppExit
}

#[derive(Debug, Clone, Copy)]
enum Statistic {
    PerformanceIfFC,
    PerformanceIfEndsNow,
    Accuracy
}

fn main() {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    let handle = Core::run(
        rx1,
        tx2,
        "ws://localhost:24050/tokens?bulkUpdates=MainPipeline,LiveTokens".to_owned()
    );

    App::run(tx1, rx2);

    handle.join().expect("Thread crashed");
}

