use std::{net::SocketAddrV4, str::FromStr, sync::Arc};

use protocol::EventHandler;
use tokio::{runtime::Handle, sync::Mutex};

mod communicate;
mod display;
mod gui;
mod input;
mod mouse_handler;
mod protocol;

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
    let disp2 = displays.clone();

    let (mut gui_handler, sender) = gui::GUIHandler::new();

    let handler = Arc::new(Mutex::new(mouse_handler::Handler::new(
        prot.clone(),
        displays.clone(),
        sender,
    )));
    let handler2 = handler.clone();

    comms.assign_updates(Box::new(client_updates)).await;

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
        })
        .await;
    });

    let handler3 = handler.clone();
    let tokio_handle = Handle::current();
    std::thread::spawn(move || {
        let input = input::MouseInputReceiver::new();

        input.mouse_movement_listener(|movement| {
            let handler = handler3.clone();
            tokio_handle.spawn(async move {
                match handler.lock().await.mouse_movement(movement).await {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Was unable to process MouseMovement: {}", e)
                    }
                }
            });
        })
    });

    gui_handler.start().await;
}
