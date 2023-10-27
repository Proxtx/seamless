use std::{net::SocketAddrV4, str::FromStr, sync::Arc};

mod communicate;
mod input;
mod protocol;

const GROUP_ID_PORT: &str = "225.0.4.16:31725";
const SENDER_PORT: u16 = 31726;

use crate::protocol::Event;

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
                println!("{}|{}", v.x, v.y);
                //let mut eng = enigo::Enigo::new();
                //eng.mouse_move_to(v.x,v.y);
            }
        })
        .await;
    });
    #[cfg(feature = "send_mouse")]
    {
        let rec = Arc::new(input::MouseInputReceiver::new());
        let rec_2 = rec.clone();
        let hand_2 = handler.clone();
        /*tokio::spawn(async move {
            rec.mouse_movement_listener(|movement| {
                let handler = handler.clone();

                tokio::spawn(async move {
                    handler.emit_event(Box::new(movement)).await.unwrap();
                });
            });
        });*/

        let mut last_packet: String = String::from("");
        let mut repeat_amount: u16 = 0;

        loop {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            let pos = rec_2.get_current_pos();
            let poz = pos.serialize();
            if poz == last_packet && repeat_amount > 10 {
                continue;
            } else if poz == last_packet {
                repeat_amount += 1
            } else {
                repeat_amount = 0;
            }
            last_packet = poz;

            hand_2.emit_event(Box::new(pos)).await.unwrap();
        }
    }

    b.wait().await;
}
