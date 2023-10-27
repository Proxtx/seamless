#[cfg(feature = "send_mouse")]
use device_query::{DeviceQuery, DeviceState};

#[cfg(feature = "send_mouse")]
pub struct MouseInputReceiver {
    mouse: DeviceState,
}

#[derive(Debug)]
pub struct MouseMovement {
    pub x: i32,
    pub y: i32,
}

impl MouseMovement {
    pub fn movement(&self) -> bool {
        if self.x != 0 && self.y != 0 {
            return true;
        }
        false
    }
}

#[cfg(feature = "send_mouse")]
impl MouseInputReceiver {
    pub fn new() -> Self {
        MouseInputReceiver {
            mouse: DeviceState::new(),
        }
    }

    pub fn mouse_movement_listener(&self, callback: impl Fn(MouseMovement) -> ()) {
        let mut last_pos = (0, 0);

        loop {
            let pos = self.mouse.get_mouse().coords;
            let comparison = self.compare_positions(&last_pos, &pos);
            if comparison.movement() {
                callback(MouseMovement { x: pos.0, y: pos.1 });
            }

            last_pos = pos;
        }
    }

    fn compare_positions(&self, point_1: &(i32, i32), point_2: &(i32, i32)) -> MouseMovement {
        MouseMovement {
            x: point_2.0 - point_1.0,
            y: point_2.1 - point_1.1,
        }
    }
}
