use std::{net::SocketAddrV4, str::FromStr};

mod communicate;

const GROUP_ID_PORT: &str = "225.0.0.1:8000";
const SENDER_PORT: u16 = 8001;

#[tokio::main]
async fn main() {
    let group_address: SocketAddrV4 =
        SocketAddrV4::from_str(GROUP_ID_PORT).expect("Invalid Group ID or Port");
    let communicate = communicate::Communicate::new(group_address, SENDER_PORT)
        .await
        .unwrap();
}
