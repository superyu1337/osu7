use std::{net::TcpStream, sync::mpsc::{Receiver, Sender}, thread::JoinHandle};

use mcp2221::Handle;
use osu7_i2c::Osu7Display;
use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};

use crate::{schema::Data, ChannelMsg, Statistic};

pub struct Core {
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    display: Option<Osu7Display<Handle>>
}

impl Core {
    pub fn run(rx: Receiver<ChannelMsg>, tx: Sender<ChannelMsg>, url: String) -> JoinHandle<()> {
        let mut instance = Core { socket: None, display: None };
        std::thread::spawn(move || {
            Self::inner(&mut instance, rx, tx, url);
        })
    }

    pub fn connect(&mut self, url: &str) {
        if let Ok((mut socket, _)) = tungstenite::connect(url) {
            socket.send(
                tungstenite::Message::Text("[acc,simulatedPp,ppIfMapEndsNow,ppIfRestFced]".into())
            ).expect("Failed to send message to websocket");

            self.socket = Some(socket);
        } else {
            self.socket = None;
        }
    }

    pub fn connect_display(&mut self) {
        let config = mcp2221::Config::default();

        if let Ok(handle) = mcp2221::Handle::open_first(&config) {
            self.display = Some(Osu7Display::new(handle, osu7_i2c::I2C_ADDR));
        } else {
            self.display = None;
        }
    }

    pub fn read_socket(&mut self) -> Option<Message> {
        self.socket
            .as_mut()
            .map(|socket| 
                socket.read().expect("Error reading message")
            )
    }

    pub fn inner(&mut self, rx: Receiver<ChannelMsg>, tx: Sender<ChannelMsg>, url: String) {
        let mut mode = Statistic::PerformanceIfEndsNow;

        loop {
            if let Ok(msg) = rx.try_recv() {
                match msg {
                    ChannelMsg::ChangeDisplayStat(new_mode) => {
                        mode = new_mode
                    },
                    ChannelMsg::AppExit => break,
                    _ => {}
                }
            }

            if self.display.is_none() {
                tx.send(ChannelMsg::DisplayConnected(false)).expect("Channel died");
                self.connect_display();

                if self.display.is_some() {
                    tx.send(ChannelMsg::DisplayConnected(true)).expect("Channel died");
                }
            }

            if self.socket.is_none() {
                tx.send(ChannelMsg::WebsocketConnected(false)).expect("Channel died");
                self.connect(&url);

                if self.socket.is_some() {
                    tx.send(ChannelMsg::WebsocketConnected(true)).expect("Channel died");
                }

                continue;
            }

            if let Some(Message::Text(bytes)) = self.read_socket() {
                let data: Data = serde_json::from_str(bytes.as_str()).unwrap();
    
                let value_to_display = match mode {
                    Statistic::PerformanceIfFC => data.pp_if_fc(),
                    Statistic::PerformanceIfEndsNow => data.pp_ends_now(),
                    Statistic::Accuracy => data.accuracy(),
                };

                let as_int = value_to_display.round() as u32;

                if let Some(disp) = &mut self.display {
                    disp.write_buffer_integer(as_int);
                    disp.commit_buffer();
                }
            }
        }
    }
}