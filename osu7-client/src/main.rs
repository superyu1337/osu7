// Dont allocate a console on windows for release builds.
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use app::App;
use core::Core;
use schema::{streamcompanion::StreamCompanionResponse, tosu::TosuResponse, OsuData};
use std::sync::mpsc;

mod app;
mod core;
mod schema;

#[derive(Debug, Clone, Copy)]
enum ChannelMsg {
    ChangeDisplayStat(Statistic),
    ChangeDisplayBrightness(Brightness),
    ChangeServer(DataProviderServer),
    DisplayConnected(bool),
    WebsocketConnected(bool),
    AppExit,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Statistic {
    PerformanceFC,
    PerformanceCurrent,
    Accuracy,
    UnstableRate,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Brightness {
    Minimum,
    Medium,
    Maximum,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum DataProviderServer {
    Tosu,
    StreamCompanion,
}

impl DataProviderServer {
    pub fn get_url(&self) -> String {
        match self {
            DataProviderServer::Tosu => String::from("ws://localhost:24050/ws"),
            DataProviderServer::StreamCompanion => {
                String::from("ws://localhost:20727/tokens?bulkUpdates=MainPipeline,LiveTokens")
            }
        }
    }

    pub fn deserialize_response(&self, data: &[u8], old_data: OsuData) -> OsuData {
        match self {
            DataProviderServer::Tosu => serde_json::from_slice::<TosuResponse>(data)
                .expect("Failed to deserialize Tosu response")
                .to_osu_data(),
            DataProviderServer::StreamCompanion => {
                serde_json::from_slice::<StreamCompanionResponse>(data)
                    .expect("Failed to deserialize StreamCompanion response")
                    .to_osu_data(old_data)
            }
        }
    }
}

fn main() {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    let handle = Core::run(rx1, tx2);

    App::run(tx1, rx2);

    handle.join().expect("Thread crashed");
}
