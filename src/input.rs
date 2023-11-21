use {
    crate::mouse_handler::Handler,
    device_query::{CallbackGuard, DeviceEvents, DeviceState, Keycode},
    std::{ops, sync::Arc},
    tokio::{runtime::Handle, sync::Mutex},
};

#[derive(Debug, Clone)]
pub struct MousePosition {
    pub x: i32,
    pub y: i32,
}

impl ops::Add<MouseMovement> for MousePosition {
    type Output = MousePosition;

    fn add(self, rhs: MouseMovement) -> Self::Output {
        MousePosition {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::AddAssign<&MouseMovement> for MousePosition {
    fn add_assign(&mut self, rhs: &MouseMovement) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl ops::AddAssign<MouseMovement> for MousePosition {
    fn add_assign(&mut self, rhs: MouseMovement) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl ops::Sub<MouseMovement> for MousePosition {
    type Output = MousePosition;

    fn sub(self, rhs: MouseMovement) -> Self::Output {
        MousePosition {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::SubAssign<&MouseMovement> for MousePosition {
    fn sub_assign(&mut self, rhs: &MouseMovement) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl ops::SubAssign<MouseMovement> for MousePosition {
    fn sub_assign(&mut self, rhs: MouseMovement) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl ops::Add<MousePosition> for MousePosition {
    type Output = MousePosition;

    fn add(self, rhs: MousePosition) -> Self::Output {
        MousePosition {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug)]
pub struct MouseMovement {
    pub x: i32,
    pub y: i32,
}

impl MouseMovement {}

pub struct MouseInputReceiver {
    mouse: DeviceState,
}

impl MouseInputReceiver {
    pub fn new() -> Self {
        MouseInputReceiver {
            mouse: DeviceState::new(),
        }
    }

    pub fn mouse_movement_listener(
        &self,
        handler: Arc<Mutex<Handler>>,
        handle: Handle,
    ) -> CallbackGuard<impl Fn(&(i32, i32))> {
        self.mouse.on_mouse_move(move |position_n_parsed| {
            let position = MousePosition {
                x: position_n_parsed.0,
                y: position_n_parsed.1,
            };
            let handler = handler.clone();
            handle.spawn(async move {
                match handler.lock().await.mouse_movement(position).await {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Was unable to process MouseMovement: {}", e)
                    }
                }
            });
        })
    }
}

struct KeyInput {}

pub struct KeyInputReceiver {
    keys: DeviceState,
}

impl KeyInputReceiver {
    pub fn new() -> Self {
        KeyInputReceiver {
            keys: DeviceState::new(),
        }
    }

    pub fn key_input_listener(
        &self,
        handle: Handle,
    ) -> (
        CallbackGuard<impl Fn(&Keycode)>,
        CallbackGuard<impl Fn(&Keycode)>,
        CallbackGuard<impl Fn(&usize)>,
        CallbackGuard<impl Fn(&usize)>,
    ) {
        let key_down_guard = self.keys.on_key_down(|key| {
            println!("Down {}", key.to_string());
        });
        let key_up_guard = self.keys.on_key_up(|key| println!("Up {}", key));
        let mouse_down_guard = self
            .keys
            .on_mouse_down(|key| println!("Mouse down {}", key));
        let mouse_up_guard = self.keys.on_mouse_up(|key| println!("Mouse up {}", key));
        (
            key_down_guard,
            key_up_guard,
            mouse_down_guard,
            mouse_up_guard,
        )
    }
}
