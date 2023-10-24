use std::net::{Ipv4Addr, UdpSocket};

const SERVICE_NAME: &'static str = "_http._tcp.local";

struct Device {
    ip: Ipv4Addr,
    port: u16,
}

pub struct Communicate {
    devices: Devices,
}

pub struct Devices {
    devices: Vec<Device>,
}

impl Devices {
    pub fn new() -> Self {
        Devices {
            devices: Vec::new(),
        }
    }

    pub async fn search(&mut self) {
        //mdns::discover::;
        let disc = mdns::discover::all(SERVICE_NAME, std::time::Duration::from_secs(15))
            .expect("err")
            .listen();

        //pin_mut!(disc);

        //Ipv4Network::with_netmask(Ipv4Addr::new(0, 0, 0, 0));
    }
}
