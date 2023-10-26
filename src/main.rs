use std::{net::SocketAddrV4, str::FromStr, sync::Arc};

mod communicate;
mod input;
mod protocol;

const GROUP_ID_PORT: &str = "225.0.4.16:31725";
const SENDER_PORT: u16 = 31726;

#[tokio::main]
async fn main() {
    let communicate = Arc::new(
        communicate::Communicate::new(SocketAddrV4::from_str(GROUP_ID_PORT).unwrap(), SENDER_PORT)
            .await
            .unwrap(),
    );
    let handler = protocol::EventHandler::new(communicate);

    let b = Arc::new(tokio::sync::Barrier::new(2));

    handler.event_listener(|event| println!("Event: {}", event.serialize()));

    handler
        .emit_event(Box::new(input::MouseMovement { x: 5, y: 10 }))
        .await
        .unwrap();

    b.wait().await;
}
