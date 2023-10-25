use std::{
    error::Error,
    fmt, io,
    net::{IpAddr, SocketAddr},
    
};
use tokio::net::UdpSocket;

#[derive(Debug)]
pub enum UdpError {
    BindingError,
    IoError(io::Error),
}

impl fmt::Display for UdpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UdpError::BindingError => {
                write!(f, "Was unable to bind to port! Is the port in use?")
            }
            UdpError::IoError(..) => {
                write!(f, "Udp Error: IO Error")
            }
        }
    }
}

impl Error for UdpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            UdpError::BindingError => None,
            UdpError::IoError(ref error) => Some(error),
        }
    }
}

impl From<io::Error> for UdpError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

pub struct UdpCommunicate {
    udp: UdpSocket,
}

impl UdpCommunicate {
    pub async fn new(port: u16) -> Result<UdpCommunicate, UdpError> {
        let udp = match UdpSocket::bind("127.0.0.1:".to_owned() + &port.to_string()).await {
            Ok(v) => v,
            Err(e) => {
                return Err(UdpError::BindingError);
            }
        };

        udp.join_multicast_v4(multiaddr, interface);

        Ok(UdpCommunicate { udp })
    }

    pub async fn send(&self, addr: SocketAddr, data: String) -> Result<(), UdpError> {
        self.udp.send_to(data.as_bytes(), addr).await?;
        Ok(())
    }

    pub fn receive(&self) -> String{
        let buf = [0; 2024];
        let (amt, src) = self.udp.
    }
}
