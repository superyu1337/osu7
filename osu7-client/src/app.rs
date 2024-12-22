use std::sync::mpsc::{Receiver, Sender};

use tao::{event::Event, event_loop::{ControlFlow, EventLoopBuilder}};
use tray_icon::{menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu}, TrayIconBuilder, TrayIconEvent};

use crate::ChannelMsg;

const ICON_BUFFER: &[u8; 5169] = include_bytes!("../asset/icon.png");

pub struct App;

#[derive(Debug)]
enum AppEvent {
    CoreMessage(ChannelMsg),
    TrayIcon(tray_icon::TrayIconEvent),
    Menu(tray_icon::menu::MenuEvent)
}

impl App {
    pub fn run(tx: Sender<ChannelMsg>, rx: Receiver<ChannelMsg>) {
        let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();

        let proxy = event_loop.create_proxy();
        std::thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                proxy
                    .send_event(AppEvent::CoreMessage(msg))
                    .expect("Failed to send CoreMessageEvent")
            }
        });

        // set a tray event handler that forwards the event and wakes up the event loop
        let proxy = event_loop.create_proxy();
        TrayIconEvent::set_event_handler(Some(move |event| {
            proxy
                .send_event(AppEvent::TrayIcon(event))
                .expect("Failed to send TrayIconEvent")
        }));

        // set a menu event handler that forwards the event and wakes up the event loop
        let proxy = event_loop.create_proxy();
        MenuEvent::set_event_handler(Some(move |event| {
            proxy
                .send_event(AppEvent::Menu(event))
                .expect("Failed to send MenuEvent");
        }));

        let tray_menu = Menu::new();

        let display_options = Submenu::new("Display", true);
        let pp_ends_now_i = CheckMenuItem::new("PP (If ends now)", false, true, None);
        let pp_if_fc_i = CheckMenuItem::new("PP (If FC)", true, false, None);
        let acc_i = CheckMenuItem::new("Accuracy", true, false, None);
        let ur_i = CheckMenuItem::new("Unstable Rate", true, false, None);

        display_options.append_items(&[
            &pp_ends_now_i,
            &pp_if_fc_i,
            &acc_i,
            &ur_i
        ]).unwrap();

        let quit_i = MenuItem::new("Quit", true, None);
        let ws_connected = CheckMenuItem::new("WebSocket Connected", false, false, None);
        let display_connected = CheckMenuItem::new("Display Connected", false, false, None);

        tray_menu.append_items(&[
            &ws_connected,
            &display_connected,
            &PredefinedMenuItem::separator(),
            &display_options,
            &PredefinedMenuItem::separator(),
            &quit_i,
        ]).unwrap();

        let mut tray_icon = None;

        let _menu_channel = MenuEvent::receiver();
        let _tray_channel = TrayIconEvent::receiver();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(tao::event::StartCause::Init) => {
                    let icon = Self::icon();

                    // We create the icon once the event loop is actually running
                    // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
                    tray_icon = Some(
                        TrayIconBuilder::new()
                            .with_menu(Box::new(tray_menu.clone()))
                            .with_tooltip("tao - awesome windowing lib")
                            .with_icon(icon)
                            .build()
                            .unwrap(),
                    );

                    // We have to request a redraw here to have the icon actually show up.
                    // Tao only exposes a redraw method on the Window so we use core-foundation directly.
                    #[cfg(target_os = "macos")]
                    unsafe {
                        use core_foundation::runloop::{CFRunLoopGetMain, CFRunLoopWakeUp};

                        let rl = CFRunLoopGetMain();
                        CFRunLoopWakeUp(rl);
                    }
                }

                Event::UserEvent(AppEvent::CoreMessage(msg)) => {
                    match msg {
                        ChannelMsg::DisplayConnected(connected) => {
                            display_connected.set_checked(connected);
                        },
                        ChannelMsg::WebsocketConnected(connected) => {
                            ws_connected.set_checked(connected);
                        },
                        _ => {},
                    }
                }

                // TODO: still needed?
                Event::UserEvent(AppEvent::TrayIcon(_event)) => {
                    //println!("{event:?}");
                }

                Event::UserEvent(AppEvent::Menu(event)) => {
                    if event.id == pp_if_fc_i.id() && pp_if_fc_i.is_checked() {
                        acc_i.set_checked(false);
                        acc_i.set_enabled(true);
                        pp_ends_now_i.set_checked(false);
                        pp_ends_now_i.set_enabled(true);

                        pp_if_fc_i.set_enabled(false);

                        tx.send(
                            ChannelMsg::ChangeDisplayStat(
                                crate::Statistic::PerformanceIfFC
                            )
                        ).expect("Channel died")
                    }

                    if event.id == pp_ends_now_i.id() && pp_ends_now_i.is_checked() {
                        acc_i.set_checked(false);
                        acc_i.set_enabled(true);
                        pp_if_fc_i.set_checked(false);
                        pp_if_fc_i.set_enabled(true);
                        ur_i.set_checked(false);
                        ur_i.set_enabled(true);

                        pp_ends_now_i.set_enabled(false);

                        tx.send(
                            ChannelMsg::ChangeDisplayStat(
                                crate::Statistic::PerformanceIfEndsNow
                            )
                        ).expect("Channel died")
                    }

                    if event.id == acc_i.id() && acc_i.is_checked() {
                        pp_ends_now_i.set_checked(false);
                        pp_ends_now_i.set_enabled(true);
                        pp_if_fc_i.set_checked(false);
                        pp_if_fc_i.set_enabled(true);
                        ur_i.set_checked(false);
                        ur_i.set_enabled(true);

                        acc_i.set_enabled(false);

                        tx.send(
                            ChannelMsg::ChangeDisplayStat(
                                crate::Statistic::Accuracy
                            )
                        ).expect("Channel died")
                    }

                    if event.id == ur_i.id() && ur_i.is_checked() {
                        pp_ends_now_i.set_checked(false);
                        pp_ends_now_i.set_enabled(true);
                        pp_if_fc_i.set_checked(false);
                        pp_if_fc_i.set_enabled(true);
                        acc_i.set_checked(false);
                        acc_i.set_enabled(true);

                        ur_i.set_enabled(false);

                        tx.send(
                            ChannelMsg::ChangeDisplayStat(
                                crate::Statistic::UnstableRate
                            )
                        ).expect("Channel died")
                    }

                    if event.id == quit_i.id() {
                        tx.send(
                            ChannelMsg::AppExit
                        ).expect("Channel died");

                        tray_icon.take();
                        *control_flow = ControlFlow::Exit;
                    }
                }

                _ => {}
            }
        })
    }

    fn icon() -> tray_icon::Icon {
        let (icon_rgba, icon_width, icon_height) = {
            let image = image::load_from_memory(ICON_BUFFER)
                .expect("Failed to decode icon")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
    }
}