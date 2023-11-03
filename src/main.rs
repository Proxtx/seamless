use std::{net::SocketAddrV4, str::FromStr, sync::Arc};

use protocol::EventHandler;
use tokio::sync::Mutex;

mod communicate;
//maybe later
//mod gui;
mod display;
mod input;
mod protocol;

use enigo::MouseControllable;

const GROUP_ID_PORT: &str = "225.0.4.16:31725";
const SENDER_PORT: u16 = 31726;

use crate::protocol::Event;

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
                Ok(v) => {}
                Err(e) => {
                    println!("Error requesting display update!");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let barr = tokio::sync::Barrier::new(2);
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

    comms.assign_updates(Box::new(client_updates)).await;

    prot.event_listener(move |v| match v {
        protocol::Events::ClientDisplays(v) => {
            let disp = disp2.clone();
            tokio::spawn(async move {
                let mut lock = disp.lock().await;
                lock.received_displays(v).unwrap();
                println!("{:?}", lock);
            });
        }
        protocol::Events::MouseMovement(v) => {
            println!("{:?}", v)
        }
        protocol::Events::RequestDisplays(v) => {
            let comms = comms.clone();
            let prot = prot2.clone();
            tokio::spawn(async move {
                let own_ip = comms.get_own_ip().await;
                match own_ip {
                    Some(v2) => {
                        if v2.ip() == &v.client_ip {
                            prot.emit_event(Box::new(
                                display::ClientDisplays::new_local().unwrap(),
                            ))
                            .await
                            .unwrap();
                        }
                    }
                    None => {
                        println!("Unable to handle own display request because own ip is unknown");
                    }
                }
            });
        }
    })
    .await;

    barr.wait().await;
}
