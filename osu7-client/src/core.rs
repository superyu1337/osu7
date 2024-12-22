use std::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
};

use mcp2221::Handle;
use osu7_i2c::{Dimming, Osu7Display};
use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};

use crate::{schema::Data, Brightness, ChannelMsg, Statistic};

pub struct Core {
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    display: Option<Osu7Display<Handle>>,
    brightness: Brightness,
}

impl Core {
    pub fn run(rx: Receiver<ChannelMsg>, tx: Sender<ChannelMsg>, url: String) -> JoinHandle<()> {
        let mut instance = Core {
            socket: None,
            display: None,
            brightness: Brightness::Medium,
        };
        std::thread::spawn(move || {
            Self::inner(&mut instance, rx, tx, url);
        })
    }

    pub fn connect(&mut self, url: &str) {
        if let Ok((mut socket, _)) = tungstenite::connect(url) {
            socket
                .send(tungstenite::Message::Text(
                    "[acc,simulatedPp,ppIfMapEndsNow,ppIfRestFced]".into(),
                ))
                .expect("Failed to send message to websocket");

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

    pub fn inner(&mut self, rx: Receiver<ChannelMsg>, tx: Sender<ChannelMsg>, url: String) {
        let mut mode = Statistic::PerformanceIfEndsNow;
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
                            disp.device().clear_display_buffer();
                            disp.commit_buffer().expect("Failed to commit buffer");
                        }
                        break;
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
                self.connect(&url);

                if self.socket.is_some() {
                    tx.send(ChannelMsg::WebsocketConnected(true))
                        .expect("Channel died");
                }
            }

            if let Some(Message::Text(bytes)) = self.read_socket() {
                let data: Data = serde_json::from_str(bytes.as_str()).unwrap();

                let value_to_display = match mode {
                    Statistic::PerformanceIfFC => data.pp_if_fc(),
                    Statistic::PerformanceIfEndsNow => data.pp_ends_now(),
                    Statistic::Accuracy => data.accuracy(),
                    Statistic::UnstableRate => data.unstable_rate(),
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
