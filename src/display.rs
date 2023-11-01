use {
    crate::{communicate::ReceiverDevice, input::MousePosition},
    display_info::DisplayInfo,
    std::{error, fmt, net::SocketAddrV4},
};

type Result<T> = std::result::Result<T, DisplayError>;

#[derive(Debug)]
pub enum DisplayError {
    DisplayFetchError,
    ClientAddError,
    OwnIpUnknown,
    InvalidMousePosition,
}

impl error::Error for DisplayError {}

impl fmt::Display for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplayError::DisplayFetchError => write!(f, "Was unable to fetch displays"),
            DisplayError::ClientAddError => {
                write!(f, "Was unable to add client to connected displays list")
            }
            DisplayError::OwnIpUnknown => write!(f, "Own Ip is unknown"),
            DisplayError::InvalidMousePosition => write!(f, "Invalid Mouse position"),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
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

    pub fn display_with(&self) -> u32 {
        let mut res = 0;
        for display in self.displays {
            res += display.width;
        }

        res
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
enum Client {
    IsSelf,
    IsNetworked(SocketAddrV4),
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Display {
    pub id: u32,
    pub client_x: i32,
    pub client_y: i32,
    pub width: u32,
    pub height: u32,
}

impl From<DisplayInfo> for Display {
    fn from(value: DisplayInfo) -> Self {
        Self {
            id: value.id,
            client_x: value.x,
            client_y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

pub struct DisplayManager {
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

    pub fn received_displays(&mut self, client_displays: ClientDisplays) -> Result<()> {
        let mut existing_client_index: Option<usize> = None;
        let client_ipv4: SocketAddrV4 = match client_displays.client {
            Client::IsNetworked(v) => v,
            Client::IsSelf => return Err(DisplayError::ClientAddError),
        };

        for (index, client) in self.clients.iter().enumerate() {
            match client.client {
                Client::IsNetworked(v) => {
                    if v == client_ipv4 {
                        existing_client_index = Some(index)
                    }
                }
                _ => {}
            }
        }

        match existing_client_index {
            Some(v) => {
                self.clients.remove(v);
            }
            None => {}
        };

        self.clients.push(client_displays);

        self.sort_client_displays()?;

        Ok(())
    }

    pub fn filter_clients(&mut self, connected_clients: &Vec<ReceiverDevice>) {
        self.clients = self
            .clients
            .drain(..)
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

    fn sort_client_displays(&mut self) -> Result<()> {
        let own_ip = match self.own_ip {
            Some(v) => v,
            None => {
                return Err(DisplayError::OwnIpUnknown);
            }
        };

        self.clients.sort_by(|a, b| {
            let a_ipv4 = match a.client {
                Client::IsNetworked(v) => v,
                Client::IsSelf => own_ip,
            };

            let b_ipv4 = match a.client {
                Client::IsNetworked(v) => v,
                Client::IsSelf => own_ip,
            };

            a_ipv4.cmp(&b_ipv4)
        });

        Ok(())
    }

    pub fn set_own_ip(&mut self, own_ip: SocketAddrV4) {
        self.own_ip = Some(own_ip);
        self.sort_client_displays().unwrap(); //this is valid because sort_client_displays only returns one possible error, which is covered by setting own_ip
    }

    pub fn get_local_mouse_position(
        &self,
        mouse_position: MousePosition,
    ) -> Result<ClientMousePosition> {
        let current_x: u32 = 0;
        let found_client: Option<&ClientDisplays> = None;
        for client in self.clients {
            let display_width = client.display_with();
            if current_x + display_width > mouse_position.x as u32 {
                found_client = Some(&client);
                break;
            }
        }

        let client = match found_client {
            Some(v) => v,
            None => {
                return Err(DisplayError::InvalidMousePosition);
            }
        };

        Ok(())
    }
}

struct ClientMousePosition {
    client: Client,
    mouse_position: MousePosition,
}
