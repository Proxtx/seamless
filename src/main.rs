use std::{net::SocketAddrV4, str::FromStr, sync::Arc};

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
    let comms =
        communicate::Communicate::new(SocketAddrV4::from_str(GROUP_ID_PORT).unwrap(), SENDER_PORT);

    barr.wait().await;
}
