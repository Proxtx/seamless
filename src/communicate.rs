use {
    std::{
        error::Error,
        fmt,
        net::{Ipv4Addr, SocketAddrV4},
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
    send: UdpSocket,
    receive: UdpSocket,
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
            send: send_socket,
            receive: receive_socket,
            multicast_addr,
        })
    }
}
