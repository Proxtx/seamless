use {
    ipnetwork::Ipv4Network,
    network_interface::{NetworkInterface, NetworkInterfaceConfig},
    std::net::{Ipv4Addr, UdpSocket},
};

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

    pub fn search(&mut self) {
        let network_interfaces = NetworkInterface::show().unwrap();

        for itf in network_interfaces.iter() {
            println!("{:?}", itf);
        }

        //Ipv4Network::with_netmask(Ipv4Addr::new(0, 0, 0, 0));
    }
}
