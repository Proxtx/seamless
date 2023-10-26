use std::{net::SocketAddrV4, str::FromStr, sync::Arc};

mod communicate;

const GROUP_ID_PORT: &str = "225.0.0.1:8000";
const SENDER_PORT: u16 = 8001;

#[tokio::main]
async fn main() {
    let group_address: SocketAddrV4 =
        SocketAddrV4::from_str(GROUP_ID_PORT).expect("Invalid Group ID or Port");
    let communicate = Arc::new(
        communicate::Communicate::new(group_address, SENDER_PORT)
            .await
            .unwrap(),
    );
    let clc = communicate.clone();

    let cs = Arc::new(tokio::sync::Barrier::new(2));
    let cs_c = cs.clone();

    tokio::spawn(async move {
        communicate
            .receive(|msg, src| {
                println!("Received: '{}' from {}", msg, src);
            })
            .await;

        cs.wait().await;
    });

    clc.send(String::from("Hello")).await.unwrap();

    cs_c.wait().await;
}
