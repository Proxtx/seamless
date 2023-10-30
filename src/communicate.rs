use {
    std::{
        error::Error,
        fmt,
        net::{Ipv4Addr, SocketAddr, SocketAddrV4},
        sync::Arc,
        time::{Duration, Instant},
    },
    tokio::{net::UdpSocket, sync::Mutex},
    uuid::Uuid,
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

#[derive(Debug, Clone, Copy)]
struct ReceiverDevice {
    pub updated: Instant,
    pub socket_addr: SocketAddrV4,
}

impl ReceiverDevice {
    pub fn new(addr: SocketAddrV4) -> Self {
        ReceiverDevice {
            updated: Instant::now(),
            socket_addr: addr,
        }
    }

    pub fn updated(&mut self) {
        self.updated = Instant::now()
    }

    pub fn decayed(&self) -> bool {
        self.updated.elapsed() > Duration::from_secs(5)
    }
}

pub struct Communicate {
    main_socket: Arc<UdpSocket>,
    global_socket: Arc<UdpSocket>,
    multicast_addr: SocketAddrV4,
    devices: Arc<Mutex<Vec<ReceiverDevice>>>,
    broadcasting_addr: bool,
    self_id: Uuid,
    self_addr: Arc<Mutex<Option<SocketAddrV4>>>,
}

impl Communicate {
    pub async fn new(multicast_addr: SocketAddrV4, main_port: u16) -> Result<Communicate> {
        let global_socket = UdpSocket::bind(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            multicast_addr.port(),
        ))
        .await?;
        global_socket.join_multicast_v4(*multicast_addr.ip(), Ipv4Addr::UNSPECIFIED)?;

        let main_socket =
            UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, main_port)).await?;

        let mut instance = Communicate {
            main_socket: Arc::new(main_socket),
            global_socket: Arc::new(global_socket),
            multicast_addr,
            devices: Arc::new(Mutex::new(Vec::new())),
            broadcasting_addr: false,
            self_id: Uuid::new_v4(),
            self_addr: Arc::new(Mutex::new(None)),
        };

        Communicate::devices_updater(&mut instance);
        Communicate::broadcast_address(&mut instance);

        Ok(instance)
    }

    pub async fn send(&self, message: String) -> Result<()> {
        for client in self.devices.lock().await.iter() {
            self.main_socket
                .send_to(message.as_bytes(), client.socket_addr)
                .await?;
        }

        Ok(())
    }

    pub async fn receive(&self, callback: impl Fn(&str, SocketAddr)) {
        let mut buf: [u8; 2024] = [0; 2024];
        loop {
            match self.main_socket.recv_from(&mut buf).await {
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

    fn devices_updater(&mut self) {
        let devices = self.devices.clone();
        let global_socket = self.global_socket.clone();
        let self_id = self.self_id.clone();
        let self_addr = self.self_addr.clone();

        tokio::spawn(async move {
            let mut buf: [u8; 2024] = [0; 2024];
            loop {
                match global_socket.recv_from(&mut buf).await {
                    Ok((amount, src)) => {
                        let buf = &mut buf[..amount];
                        let text = match std::str::from_utf8(buf) {
                            Ok(v) => v,
                            Err(e) => {
                                println!("Unable to read devices updater string. {}", e);
                                continue;
                            }
                        };
                        let uuid = match Uuid::parse_str(text) {
                            Ok(v) => v,
                            Err(e) => {
                                println!("Received a wrong uuid! {}", e);
                                continue;
                            }
                        };
                        if uuid == self_id {
                            let mut lock = self_addr.lock().await;
                            match src {
                                SocketAddr::V4(src) => *lock = Some(src),
                                _ => println!("Received self_address as V6! Invalid!"),
                            }
                            continue;
                        }
                        let mut devices = devices.lock().await;
                        let clean_devices = Communicate::clean_devices(devices.to_vec());
                        *devices = clean_devices;

                        let mut found = false;
                        for device in devices.iter_mut() {
                            if device.socket_addr.to_string() == src.to_string() {
                                device.updated();
                                found = true;
                            }
                        }

                        match (found, src) {
                            (false, SocketAddr::V4(v)) => {
                                devices.push(ReceiverDevice::new(v));
                            }
                            (false, _) => {
                                println!("Got a message from an IPv6 sender")
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        println!("Error reading Socket: {}", e)
                    }
                }
            }
        });
    }

    fn clean_devices(devices: Vec<ReceiverDevice>) -> Vec<ReceiverDevice> {
        devices.into_iter().filter(|v| !v.decayed()).collect()
    }

    fn broadcast_address(&mut self) {
        if self.broadcasting_addr {
            return;
        }

        self.broadcasting_addr = true;

        let sender_clone = self.main_socket.clone();
        let addr = self.multicast_addr.clone();
        let id = self.self_id.clone();

        tokio::spawn(async move {
            loop {
                if let Err(e) = sender_clone
                    .send_to(format!("{}", id).as_bytes(), addr)
                    .await
                {
                    println!("Error broadcasting own address: {}", e)
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }
}
