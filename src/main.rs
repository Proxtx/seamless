mod communicate;
mod input;

fn main() {
    let mut d = communicate::Devices::new();
    d.search();

    #[cfg(feature = "send")]
    {
        use input::{MouseInputReceiver, MouseMovement};

        let receiver = MouseInputReceiver::new();
        receiver.mouse_movement_listener(|movement: MouseMovement| {
            //println!("{:?}", movement);
        })
    }
}
