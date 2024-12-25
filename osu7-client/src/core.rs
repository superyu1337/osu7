use std::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
};

use mcp2221::Handle;
use osu7_i2c::{Dimming, Display, Osu7Display};
use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};

use crate::{schema::OsuData, Brightness, ChannelMsg, DataProviderServer, Statistic};

const STREAMCOMPANION_FIRSTMSG: &str = r#"["acc","ppIfMapEndsNow","ppIfRestFced","unstableRate"]"#;

pub struct Core {
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    display: Option<Osu7Display<Handle>>,
    brightness: Brightness,
    server: DataProviderServer,
    data: OsuData,
}

impl Core {
    pub fn run(rx: Receiver<ChannelMsg>, tx: Sender<ChannelMsg>) -> JoinHandle<()> {
        let mut instance = Core {
            socket: None,
            display: None,
            brightness: Brightness::Medium,
            server: DataProviderServer::Tosu,
            data: OsuData::default(),
        };
        std::thread::spawn(move || {
            Self::inner(&mut instance, rx, tx);
        })
    }

    pub fn connect(&mut self) {
        let url = self.server.get_url();

        if let Ok((mut socket, _)) = tungstenite::connect(url) {
            if self.server == DataProviderServer::StreamCompanion {
                socket
                    .send(tungstenite::Message::Text(STREAMCOMPANION_FIRSTMSG.into()))
                    .expect("Failed to send message to websocket");
            }

            self.socket = Some(socket);
        } else {
            self.socket = None;
        }
    }

    fn get_dimming(&self) -> Dimming {
        match self.brightness {
            Brightness::Minimum => Dimming::BRIGHTNESS_MIN,
            Brightness::Medium => Dimming::BRIGHTNESS_8_16,
            Brightness::Maximum => Dimming::BRIGHTNESS_MAX,
        }
    }

    pub fn connect_display(&mut self) {
        let config = mcp2221::Config::default();

        if let Ok(handle) = mcp2221::Handle::open_first(&config) {
            self.display = Some(Osu7Display::new(handle, osu7_i2c::I2C_ADDR));

            let dimming = self.get_dimming();

            let disp = self.display.as_mut().unwrap();
            disp.initialize();
            disp.device().set_dimming(dimming).unwrap();
        } else {
            self.display = None;
        }
    }

    pub fn read_socket(&mut self) -> Option<Message> {
        if let Some(ws) = &mut self.socket {
            ws.read().ok()
        } else {
            None
        }
    }

    pub fn inner(&mut self, rx: Receiver<ChannelMsg>, tx: Sender<ChannelMsg>) {
        let mut mode = Statistic::PerformanceCurrent;
        let mut last_brightness = self.brightness;

        loop {
            if let Ok(msg) = rx.try_recv() {
                match msg {
                    ChannelMsg::ChangeDisplayStat(new_mode) => mode = new_mode,
                    ChannelMsg::ChangeDisplayBrightness(brightness) => {
                        self.brightness = brightness;
                    }
                    ChannelMsg::AppExit => {
                        if let Some(disp) = &mut self.display {
                            disp.device()
                                .set_display(Display::OFF)
                                .expect("Failed to turn off display");
                        }

                        tx.send(ChannelMsg::AppExit).expect("Channel died");
                    }
                    ChannelMsg::ChangeServer(new_server) => {
                        self.server = new_server;
                        self.socket = None;
                        tx.send(ChannelMsg::WebsocketConnected(false))
                            .expect("Channel died");
                    }
                    _ => {}
                }
            }

            if last_brightness != self.brightness {
                let dimming = self.get_dimming();

                if let Some(disp) = &mut self.display {
                    disp.device()
                        .set_dimming(dimming)
                        .expect("failed to set dimming");
                    last_brightness = self.brightness;
                }
            }

            if self.display.is_none() {
                self.connect_display();

                if self.display.is_some() {
                    tx.send(ChannelMsg::DisplayConnected(true))
                        .expect("Channel died");
                }
            }

            if self.socket.is_none() {
                self.connect();

                if self.socket.is_some() {
                    tx.send(ChannelMsg::WebsocketConnected(true))
                        .expect("Channel died");
                }
            }

            if let Some(Message::Text(bytes)) = self.read_socket() {
                let new_data: OsuData = self
                    .server
                    .deserialize_response(bytes.as_bytes(), self.data);
                self.data = new_data;

                let value_to_display = match mode {
                    Statistic::PerformanceFC => self.data.pp_fc(),
                    Statistic::PerformanceCurrent => self.data.pp_current(),
                    Statistic::Accuracy => self.data.accuracy(),
                    Statistic::UnstableRate => self.data.unstable_rate(),
                };

                match mode {
                    Statistic::Accuracy => {
                        let v = value_to_display as f32;

                        if let Some(disp) = &mut self.display {
                            disp.write_buffer_float(v);
                            if disp.commit_buffer().is_err() {
                                self.display = None;
                                tx.send(ChannelMsg::DisplayConnected(false))
                                    .expect("Channel died");
                            }
                        }
                    }
                    _ => {
                        let v = value_to_display.round() as u32;

                        if let Some(disp) = &mut self.display {
                            disp.write_buffer_integer(v);
                            if disp.commit_buffer().is_err() {
                                self.display = None;
                                tx.send(ChannelMsg::DisplayConnected(false))
                                    .expect("Channel died");
                            }
                        }
                    }
                }
            } else {
                self.socket = None;
                tx.send(ChannelMsg::WebsocketConnected(false))
                    .expect("Channel died");

                if let Some(disp) = &mut self.display {
                    disp.device().clear_display_buffer();
                    if disp.write_buffer_osu7().is_err() || disp.commit_buffer().is_err() {
                        self.display = None;
                        tx.send(ChannelMsg::DisplayConnected(false))
                            .expect("Channel died");
                    }
                }
            }
        }
    }
}
