mod input;

use input::{MouseInputReceiver, MouseMovement};

fn main() {
    let receiver = MouseInputReceiver::new();
    receiver.mouse_movement_listener(|movement: MouseMovement| {
        println!("{:?}", movement);
    })
}
