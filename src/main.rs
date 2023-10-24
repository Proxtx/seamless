mod communicate;
mod input;
use async_std::stream::StreamExt;
use tokio::pin;

const SERVICE_NAME: &'static str = "_seamless._udp.local";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    std::net::UdpSocket::bind("127.0.0.1:34254")?;

    //send

    let responder = libmdns::Responder::new().unwrap();
    let _svc = responder.register(
        "_seamless._udp".to_owned(),
        "Seamless UDP Server".to_owned(),
        34254,
        &["path=/"],
    );

    //listen
    let stream = mdns::discover::all(SERVICE_NAME, std::time::Duration::from_secs(15))
        .unwrap()
        .listen();
    pin!(stream);
    while let Some(Ok(response)) = stream.next().await {
        let addr = response.ip_addr();

        if let Some(addr) = addr {
            println!("found cast device at {}", addr);
        } else {
            println!("cast device does not advertise address");
        }
    }
    //Ok(())

    //let mut d = communicate::Devices::new();
    //d.search();

    println!("return");

    /*#[cfg(feature = "send")]
    {
        use input::{MouseInputReceiver, MouseMovement};

        let receiver = MouseInputReceiver::new();
        receiver.mouse_movement_listener(|movement: MouseMovement| {
            //println!("{:?}", movement);
        })
    }*/

    Ok(())
}
