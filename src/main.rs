use std::{net::SocketAddrV4, str::FromStr, sync::Arc};

mod communicate;
mod input;
mod protocol;

const GROUP_ID_PORT: &str = "225.0.4.16:31725";
const SENDER_PORT: u16 = 31726;

#[tokio::main]
async fn main() {}
