use std::{sync::mpsc::Receiver, thread::JoinHandle};

use osu7_i2c::{i2c_mock, Osu7Display};
use tungstenite::Message;

use crate::{schema::Data, ChannelMsg, Statistic};

pub struct Core;

impl Core {
    pub fn run(rx: Receiver<ChannelMsg>, url: String) -> JoinHandle<()> {
        std::thread::spawn(|| {
            Self::inner(rx, url);
        })
    }

    pub fn inner(rx: Receiver<ChannelMsg>, url: String) {
        let (mut socket, _) = tungstenite::connect(url)
            .expect("Can't connect");
        socket.send(
            tungstenite::Message::Text("[acc,simulatedPp,ppIfMapEndsNow,ppIfRestFced]".into())
        ).expect("Failed to send message to websocket");

        let mut mode = Statistic::PerformanceIfEndsNow;

        let config = mcp2221::Config::default();
        let i2c = mcp2221::Handle::open_first(&config).unwrap();
        let mut display = Osu7Display::new(i2c, osu7_i2c::I2C_ADDR);

        loop {
            if let Ok(msg) = rx.try_recv() {
                match msg {
                    ChannelMsg::ChangeDisplayStat(new_mode) => {
                        mode = new_mode
                    },
                    ChannelMsg::Exit => break,
                }
            }

            let msg = socket.read().expect("Error reading message");
                
            if let Message::Text(bytes) = msg {
                let data: Data = serde_json::from_str(bytes.as_str()).unwrap();

                let value_to_display = match mode {
                    Statistic::PerformanceIfFC => data.pp_if_fc(),
                    Statistic::PerformanceIfEndsNow => data.pp_ends_now(),
                    Statistic::Accuracy => data.accuracy(),
                };

                let as_int = value_to_display.round() as u32;
                display.write_buffer_integer(as_int);
                display.commit_buffer();

                println!("{:#?}: {}", mode, as_int);
            }
        }
    }
}