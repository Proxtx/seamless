use std::{net::SocketAddrV4, str::FromStr, sync::Arc};

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
    let prot = protocol::EventHandler::new(comms2);
    prot.event_listener(|v| match v {
        protocol::Events::ClientDisplays(v) => {
            println!("{:?}", v)
        }
        protocol::Events::MouseMovement(v) => {
            println!("{:?}", v)
        }
    })
    .await;

    barr.wait().await;
}
