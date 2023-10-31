use {
    crate::communicate::ReceiverDevice,
    display_info::DisplayInfo,
    std::{error, fmt, net::SocketAddrV4, sync::Arc},
    tokio::sync::Mutex,
};

type Result<T> = std::result::Result<T, DisplayError>;

#[derive(Debug)]
pub enum DisplayError {
    DisplayFetchError,
}

impl error::Error for DisplayError {}

impl fmt::Display for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplayError::DisplayFetchError => write!(f, "Was unable to fetch displays"),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ClientDisplays {
    client: Client,
    displays: Vec<Display>, //ordered
}

impl ClientDisplays {
    pub fn new_local() -> Result<Self> {
        let mut info = match DisplayInfo::all() {
            Ok(v) => v,
            Err(_e) => return Err(DisplayError::DisplayFetchError),
        };
        info.sort_by(|a, b| a.id.cmp(&b.id));
        let mut client_displays: Vec<Display> = Vec::new();
        for display in info.into_iter() {
            client_displays.push(display.into());
        }

        Ok(Self {
            client: Client::IsSelf,
            displays: client_displays,
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
enum Client {
    IsSelf,
    IsNetworked(SocketAddrV4),
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Display {
    pub id: u32,
    pub client_x: i32,
    pub client_y: i32,
    pub with: u32,
    pub height: u32,
}

impl From<DisplayInfo> for Display {
    fn from(value: DisplayInfo) -> Self {
        Self {
            id: value.id,
            client_x: value.x,
            client_y: value.y,
            with: value.width,
            height: value.height,
        }
    }
}

struct DisplayManager {
    clients: Vec<ClientDisplays>,
    own_ip: Option<SocketAddrV4>,
}

impl DisplayManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            clients: vec![ClientDisplays::new_local()?],
            own_ip: None,
        })
    }

    pub fn received_displays(&mut self, client_displays: ClientDisplays) {}

    pub fn filter_clients(&mut self, connected_clients: &Vec<ReceiverDevice>) {
        self.clients = *self
            .clients
            .into_iter()
            .filter(|v| match v.client {
                Client::IsSelf => return true,
                Client::IsNetworked(addr) => {
                    for client in connected_clients {
                        if client.socket_addr == addr {
                            return true;
                        }
                    }
                    return false;
                }
            })
            .collect();
    }

    pub fn set_own_ip(&mut self, own_ip: SocketAddrV4) {
        self.own_ip = Some(own_ip);
    }
}
