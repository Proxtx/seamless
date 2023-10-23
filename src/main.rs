fn main() {
    #[cfg(feature = "send")]
    {
        mod input;
        use input::{MouseInputReceiver, MouseMovement};

        let receiver = MouseInputReceiver::new();
        receiver.mouse_movement_listener(|movement: MouseMovement| {
            println!("{:?}", movement);
        })
    }
}
