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
    let handler = Arc::new(protocol::EventHandler::new(communicate));

    let b = Arc::new(tokio::sync::Barrier::new(2));

    let d_h = handler.clone();
    tokio::spawn(async move {
        d_h.event_listener(|event| match event {
            protocol::Events::MouseMovement(v) => {
                //let eng = enigo::Enigo::new()
                //eng.mouse_move_relative(v.x,v.y)
            }
        })
        .await;
    });
    let rec = input::MouseInputReceiver::new();
    rec.mouse_movement_listener(|movement| {
        let handler = handler.clone();

        tokio::spawn(async move {
            handler.emit_event(Box::new(movement)).await.unwrap();
        });
    });

    b.wait().await;
}
