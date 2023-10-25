use {
    socket2::{Domain, Protocol, SockAddr, Socket, Type},
    std::{
        io,
        net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket},
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Barrier,
        },
        thread::{sleep, JoinHandle},
        time::Duration,
    },
};

pub const PORT: u16 = 31435;

pub fn get_glb_ipv4() -> IpAddr {
    Ipv4Addr::new(224, 0, 0, 123).into()
}

pub fn get_glb_ipv6() -> IpAddr {
    Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0x0123).into()
}

fn multicast_listener(
    response: &'static str,
    client_done: Arc<AtomicBool>,
    addr: SocketAddr,
) -> JoinHandle<()> {
    let server_barrier = Arc::new(Barrier::new(2));
    let client_barrier = Arc::clone(&server_barrier);

    let join_handle = std::thread::Builder::new()
        .name(format!("{}:server", response))
        .spawn(move || {
            let listener = join_multicast(addr)?;
            println!("{}:server: joined: {}", response, addr);

            server_barrier.wait();
            println!("{}:server: is ready", response);

            while !client_done.load(std::sync::atomic::Ordering::Relaxed) {
                let mut buf = [0u8; 64];

                match listener.recv_from(&mut buf) {
                    Ok((len, remote_addr)) => {
                        let data = &buf[..len];

                        println!(
                            "{}:server: got data: {} from: {:?}",
                            response,
                            String::from_utf8_lossy(data),
                            remote_addr
                        );

                        let responder = new_socket(&remote_addr).expect("failed to respond");

                        responder
                            .send_to(response.as_bytes(), &remote_addr)
                            .expect("failed to respond");

                        println!("{}:server: sent respond to: {:?}", response, remote_addr);
                    }
                    Err(err) => {
                        println!("{}:server: got an error: {}", response, err)
                    }
                }
            }

            println!("{}:server: client is done", response);
        })
        .unwrap();

    client_barrier.wait();
    join_handle
}

struct NotifyServer(Arc<AtomicBool>);
impl Drop for NotifyServer {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

fn test_multicast(test: &'static str, addr: IpAddr) {
    /*assert!(addr.is_multicast());
    let addr = SocketAddr::new(addr, PORT);

    let client_done = Arc::new(AtomicBool::new(false));
    let t = NotifyServer(Arc::clone(&client_done));

    multicast_listener(test, client_done, addr);

    println!("{}:client: running", test);
    drop(t);*/

    let addr = SocketAddr::new(addr, PORT);

    println!("{}:client: running", test);

    let message = b"Hello from client!";
    let socket = new_sender(&addr).expect("could not create sender");
    socket
        .send_to(message, &SockAddr::from(addr))
        .expect("could not send_to!");
}

fn new_socket(addr: &SocketAddr) -> io::Result<Socket> {
    let domain = if addr.is_ipv4() {
        Domain::IPV4
    } else {
        Domain::IPV6
    };

    let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;

    socket.set_read_timeout(Some(Duration::from_millis(100)))?;

    Ok(socket)
}

fn join_multicast(addr: SocketAddr) -> io::Result<Socket> {
    let ip_addr = addr.ip();

    let socket = new_socket(&addr)?;

    match ip_addr {
        IpAddr::V4(ref mdns_v4) => {
            socket.join_multicast_v4(mdns_v4, &Ipv4Addr::new(0, 0, 0, 0))?;
        }

        IpAddr::V6(ref mdns_v6) => {
            socket.join_multicast_v6(mdns_v6, 0)?;
            socket.set_only_v6(true)?;
        }
    }

    socket.bind(&SockAddr::from(addr))?;

    Ok(socket)
}

fn new_sender(addr: &SocketAddr) -> io::Result<Socket> {
    let socket = new_socket(addr)?;
    if addr.is_ipv4() {
        socket.bind(&SockAddr::from(SocketAddr::new(
            Ipv4Addr::new(0, 0, 0, 0).into(),
            0,
        )))?
    } else {
        socket.bind(&SockAddr::from(SocketAddr::new(
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(),
            0,
        )))?
    }

    Ok(socket)
}

pub fn main() {}

#[test]
fn test_ipv4_multicast() {
    test_multicast("ipv4", get_glb_ipv4());
}

#[test]
fn test_ipv6_multicast() {
    test_multicast("ipv6", get_glb_ipv6());
}
