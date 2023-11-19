use {
    crate::{communicate::ReceiverDevice, input::MousePosition},
    display_info::DisplayInfo,
    std::{error, fmt, net::SocketAddrV4, ptr},
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

pub enum Edge {
    Left,
    Right,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ClientDisplays {
    pub client: Client,
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

    pub fn on_horizontal_edge(&self, mouse_position: &MousePosition) -> Option<Edge> {
        if self.displays.len() == 0 {
            return None;
        }
        let display_left = self.displays.first().unwrap(); //this is ok because we checked the length of client_displays first
        if mouse_position.x <= display_left.client_x {
            return Some(Edge::Left);
        }

        let display_right = self.displays.last().unwrap(); //this is ok because we checked the length of client_displays first
        if mouse_position.x >= display_right.client_x + display_left.width as i32 - 1 {
            return Some(Edge::Right);
        }

        None
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Client {
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

impl Display {
    pub fn contains(&self, mouse_position: &MousePosition) -> bool {
        (mouse_position.x >= self.client_x && mouse_position.x <= self.client_x + self.width as i32)
            && (mouse_position.y >= self.client_y
                && mouse_position.y <= self.client_y + self.height as i32)
    }
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

#[derive(Debug)]
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

    pub fn get_own_client_displays_index(&self) -> Option<usize> {
        for (index, client) in self.clients.iter().enumerate() {
            match client.client {
                Client::IsSelf => {
                    return Some(index);
                }
                Client::IsNetworked(_) => {}
            }
        }

        None
    }

    pub fn is_on_edge(
        &self,
        mouse_position: &MousePosition,
        client_index: usize,
    ) -> Result<Option<Edge>> {
        let client = self.clients.get(client_index);
        match client {
            Some(v) => {
                let edge = v.on_horizontal_edge(mouse_position);
                match edge {
                    Some(v) => {
                        match (client_index > 0, client_index < self.clients.len() - 1, &v) {
                            (true, _, Edge::Left) => Ok(Some(v)),
                            (_, true, Edge::Right) => Ok(Some(v)),
                            _ => Ok(None),
                        }
                    }
                    None => Ok(None),
                }
            }
            None => Err(DisplayError::DisplayFetchError),
        }
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

            let b_ipv4 = match b.client {
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
        mouse_position: &MousePosition,
    ) -> Result<ClientMousePosition> {
        struct FoundDisplay<'a> {
            pub client_displays: &'a ClientDisplays,
            pub display: &'a Display,
            pub display_position: MousePosition,
        }

        impl<'a> FoundDisplay<'a> {
            pub fn new(
                client_displays: &ClientDisplays,
                display_index: usize,
                display_position: MousePosition,
            ) -> Result<FoundDisplay> {
                let display = client_displays.displays.get(display_index).unwrap(); //is ok because i just gave you my index
                if display_position.x < 0
                    || display_position.y < 0
                    || display.width < display_position.x as u32
                    || display.height < display_position.y as u32
                {
                    return Err(DisplayError::InvalidMousePosition);
                }

                Ok(FoundDisplay {
                    client_displays,
                    display,
                    display_position,
                })
            }

            pub fn get_local_position(&self) -> MousePosition {
                MousePosition {
                    x: self.display.client_x + self.display_position.x,
                    y: self.display.client_y + self.display_position.y,
                }
            }
        }

        let mut current_x: u32 = 0;
        let mut found_client: Option<FoundDisplay> = None;
        'client_loop: for client in self.clients.iter() {
            for (index, display) in client.displays.iter().enumerate() {
                if current_x + display.width > mouse_position.x as u32 {
                    found_client = Some(FoundDisplay::new(
                        client,
                        index,
                        MousePosition {
                            x: mouse_position.x - current_x as i32,
                            y: mouse_position.y,
                        },
                    )?);
                    break 'client_loop;
                }
                current_x += display.width;
            }
        }

        let result = match found_client {
            Some(v) => v,
            None => {
                return Err(DisplayError::InvalidMousePosition);
            }
        };

        Ok(ClientMousePosition {
            client: result.client_displays.client.clone(),
            mouse_position: result.get_local_position(),
        })
    }

    pub fn get_global_mouse_position(
        &self,
        local_position: MousePosition,
    ) -> Result<MousePosition> {
        for client in self.clients.iter() {
            match client.client {
                Client::IsNetworked(_) => {}
                Client::IsSelf => {
                    for display in client.displays.iter() {
                        if display.contains(&local_position) {
                            return Ok(self.get_display_position(&display)?
                                + MousePosition {
                                    x: local_position.x - display.client_x,
                                    y: local_position.y - display.client_y,
                                });
                        }
                    }
                }
            }
        }

        Err(DisplayError::InvalidMousePosition)
    }

    fn get_display_position(&self, display: &Display) -> Result<MousePosition> {
        let mut total_x: u32 = 0;
        for client in self.clients.iter() {
            for client_display in client.displays.iter() {
                if ptr::eq(client_display, display) {
                    return Ok(MousePosition {
                        x: total_x as i32,
                        y: 0,
                    });
                }
                total_x += client_display.width;
            }
        }

        Err(DisplayError::DisplayFetchError)
    }

    pub fn get_missing_displays<'a>(
        &self,
        devices: &'a Vec<ReceiverDevice>,
    ) -> Vec<&'a ReceiverDevice> {
        let mut result = Vec::new();

        for device in devices {
            let mut found = false;

            for client in self.clients.iter() {
                match client.client {
                    Client::IsNetworked(ip) => {
                        if ip == device.socket_addr {
                            found = true;
                            break;
                        }
                    }
                    Client::IsSelf => {}
                }
            }

            if !found {
                result.push(device);
            }
        }

        result
    }
}

#[derive(Debug)]
pub struct ClientMousePosition {
    pub client: Client,
    pub mouse_position: MousePosition,
}
