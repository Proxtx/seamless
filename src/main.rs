use std::{net::SocketAddrV4, str::FromStr, sync::Arc};

use gui::GUI;
use protocol::EventHandler;
use tokio::{runtime::Handle, sync::Mutex};

mod communicate;
mod display;
mod gui;
mod input;
mod key_handler;
mod mouse_handler;
mod protocol;
use std::env;

const GROUP_ID_PORT: &str = "225.0.4.16:31725";
const SENDER_PORT: u16 = 31726;

struct ClientUpdates {
    displays: Arc<Mutex<display::DisplayManager>>,
    event_handler: Arc<EventHandler>,
}

#[async_trait::async_trait]
impl communicate::ClientUpdates for ClientUpdates {
    async fn update(&self, devices: &Vec<communicate::ReceiverDevice>) {
        let lock = self.displays.lock().await;
        let missing_displays = lock.get_missing_displays(devices);
        for device in missing_displays {
            match self
                .event_handler
                .emit_event(Box::new(protocol::RequestDisplays {
                    client_ip: device.socket_addr.ip().clone(),
                }))
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    println!("Error requesting display update: {}", e);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let own_path;
    match args.get(0) {
        Some(v) => {
            own_path = v;
        }
        None => {
            panic!("Unable to find own path!");
        }
    }
    match args.get(1) {
        Some(v) => {
            if v == "gui" {
                GUI::new();
            }
        }
        None => {}
    }

    let displays = Arc::new(Mutex::new(display::DisplayManager::new().unwrap()));
    let comms = Arc::new(
        communicate::Communicate::new(
            SocketAddrV4::from_str(GROUP_ID_PORT).unwrap(),
            SENDER_PORT,
            displays.clone(),
        )
        .await
        .unwrap(),
    );

    let comms2 = comms.clone();
    let prot = Arc::new(protocol::EventHandler::new(comms2));
    let client_updates = ClientUpdates {
        displays: displays.clone(),
        event_handler: prot.clone(),
    };
    let prot2 = prot.clone();
    let prot3 = prot.clone();
    let disp2 = displays.clone();

    let (mut gui_process_manager, gui_handler) = gui::GUIProcessManager::new(own_path.clone());

    let handler = Arc::new(Mutex::new(mouse_handler::Handler::new(
        prot.clone(),
        displays.clone(),
        Arc::new(gui_handler),
    )));
    let handler2 = handler.clone();

    comms.assign_updates(Box::new(client_updates)).await;

    let key_handler = Arc::new(Mutex::new(key_handler::Handler::new()));

    tokio::spawn(async move {
        prot.event_listener(move |v| match v {
            protocol::Events::ClientDisplays(v) => {
                let disp = disp2.clone();
                tokio::spawn(async move {
                    let mut lock = disp.lock().await;
                    match lock.received_displays(v) {
                        Err(e) => {
                            println!("Unable to add received display: {}", e);
                        }
                        _ => {}
                    };
                    println!("{:?}", lock);
                });
            }
            protocol::Events::MouseMovement(v) => {
                let handler = handler2.clone();
                tokio::spawn(async move {
                    match handler.lock().await.set_current_position(v).await {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Error setting current position: {}", e)
                        }
                    }
                });
            }
            protocol::Events::RequestDisplays(v) => {
                let comms = comms.clone();
                let prot = prot2.clone();
                println!("got display request");
                tokio::spawn(async move {
                    let own_ip = comms.get_own_ip().await;
                    match own_ip {
                        Some(v2) => {
                            if v2.ip() == &v.client_ip {
                                let own_displays = match display::ClientDisplays::new_local() {
                                    Ok(v) => v,
                                    Err(e) => {
                                        println!("Error generating own display: {}", e);
                                        return;
                                    }
                                };
                                match prot.emit_event(Box::new(own_displays)).await {
                                    Err(e) => {
                                        println!("Unable to send own display: {}", e)
                                    }
                                    _ => {}
                                }
                            }
                        }
                        None => {
                            println!(
                                "Unable to handle own display request because own ip is unknown"
                            );
                        }
                    }
                });
            }
            protocol::Events::KeyInput(input) => {
                let key_handler = key_handler.clone();
                tokio::spawn(async move {
                    match key_handler.lock().await.key_input(input) {
                        Err(e) => {
                            println!("Error sending keys: {}", e)
                        }
                        _ => {}
                    }
                });
            }
        })
        .await;
    });

    let handler3 = handler.clone();
    let mouse_input = input::MouseInputReceiver::new();
    let _gld = mouse_input.mouse_movement_listener(handler3, Handle::current());

    let key_input = input::KeyInputReceiver::new();
    let _gld2 = key_input.key_input_listener(prot3, Handle::current());

    gui_process_manager.listen().await;
}
