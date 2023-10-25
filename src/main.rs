use std::{net::SocketAddrV4, str::FromStr};

use {
    std::net::{Ipv4Addr, SocketAddr},
    tokio::net::UdpSocket,
};

const GROUP_ID: &str = "225.0.0.1";
const GROUP_ID_PORT: &str = "225.0.0.1:8000";

#[tokio::main]
async fn main() {
    let t = std::sync::Arc::new(tokio::sync::Barrier::new(2));
    let t2 = t.clone();
    tokio::spawn(async move {
        receiver().await;
        t.wait().await;
    });

    sender().await;

    t2.wait().await;
}

async fn sender() {
    let socket = UdpSocket::bind("0.0.0.0:8001").await.unwrap();
    socket
        .send_to(
            b"Hello whats up",
            SocketAddrV4::from_str(GROUP_ID_PORT).unwrap(),
        )
        .await
        .unwrap();
}

async fn receiver() {
    let socket = UdpSocket::bind("0.0.0.0:8000").await.unwrap();
    socket
        .join_multicast_v4(Ipv4Addr::from_str(GROUP_ID).unwrap(), Ipv4Addr::UNSPECIFIED)
        .unwrap();

    let mut buf = [0; 2024];
    loop {
        let (amt, src) = socket.recv_from(&mut buf).await.unwrap();

        let buf = &mut buf[..amt];
        let buf = &mut buf[..amt];
        buf.reverse();
        let buf = &mut buf[..amt];
        buf.reverse();

        println!("received: {}", std::str::from_utf8(buf).unwrap());
    }
}
