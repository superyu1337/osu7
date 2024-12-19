use app::App;
use core::Core;
use std::sync::mpsc;

mod app;
mod core;
mod schema;

#[derive(Debug, Clone, Copy)]
enum ChannelMsg {
    ChangeDisplayStat(Statistic),
    Exit
}

#[derive(Debug, Clone, Copy)]
enum Statistic {
    PerformanceIfFC,
    PerformanceIfEndsNow,
    Accuracy
}

fn main() {
    let (tx, rx) = mpsc::channel();

    let handle = Core::run(
        rx,
        "ws://127.0.0.1:24050/tokens?bulkUpdates=MainPipeline,LiveTokens".to_owned()
    );

    App::run(tx);

    handle.join().expect("Thread crashed");
}

