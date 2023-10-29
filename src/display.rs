use display_info::DisplayInfo;
use std::net::SocketAddrV4;

#[derive(serde::Serialize)]
struct ClientDisplays {
    client: Client,
    displays: Vec<Display>, //ordered
}

#[derive(serde::Serialize)]
enum Client {
    IsSelf,
    IsNetworked(SocketAddrV4),
}

#[derive(serde::Serialize)]
struct Display {
    pub id: u32,
    pub client_x: i32,
    pub client_y: i32,
    pub with: u32,
    pub height: u32,
}
