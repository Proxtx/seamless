use {
    std::{
        error::Error,
        fmt,
        net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    },
    tokio::net::UdpSocket,
};

type Result<T> = std::result::Result<T, CommunicateError>;

#[derive(Debug)]
pub enum CommunicateError {
    SocketCreationError(std::io::Error),
}

impl fmt::Display for CommunicateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            CommunicateError::SocketCreationError(ref err) => {
                write!(f, "IO Error: {}", err.to_string())
            }
        }
    }
}

impl Error for CommunicateError {}

impl From<std::io::Error> for CommunicateError {
    fn from(value: std::io::Error) -> Self {
        CommunicateError::SocketCreationError(value)
    }
}

pub struct Communicate {
    send_socket: UdpSocket,
    receive_socket: UdpSocket,
    multicast_addr: SocketAddrV4,
}

impl Communicate {
    pub async fn new(multicast_addr: SocketAddrV4, sender_port: u16) -> Result<Communicate> {
        let receive_socket = UdpSocket::bind(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            multicast_addr.port(),
        ))
        .await?;
        receive_socket.join_multicast_v4(*multicast_addr.ip(), Ipv4Addr::UNSPECIFIED)?;

        let send_socket =
            UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, sender_port)).await?;

        Ok(Communicate {
            send_socket,
            receive_socket,
            multicast_addr,
        })
    }

    pub async fn send(&self, message: String) -> Result<usize> {
        Ok(self
            .send_socket
            .send_to(message.as_bytes(), &self.multicast_addr)
            .await?)
    }

    pub async fn receive(&self, callback: impl Fn(&str, SocketAddr)) {
        let mut buf: [u8; 2024] = [0; 2024];
        loop {
            match self.receive_socket.recv_from(&mut buf).await {
                Ok((amount, socket_addr)) => {
                    let buf = &mut buf[..amount];
                    match std::str::from_utf8(buf) {
                        Ok(msg) => callback(msg, socket_addr),
                        Err(e) => {
                            println!("Error converting buffer to String: {}", e)
                        }
                    }
                }
                Err(e) => {
                    println!("Error receiving from socket: {}", e);
                }
            }
        }
    }
}
