use {
    device_query::{DeviceQuery, DeviceState},
    std::{ops, time::Duration},
};

pub struct MouseInputReceiver {
    mouse: DeviceState,
}

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

impl MouseMovement {
    pub fn movement(&self) -> bool {
        if self.x != 0 && self.y != 0 {
            return true;
        }
        false
    }
}

impl MouseInputReceiver {
    pub fn new() -> Self {
        MouseInputReceiver {
            mouse: DeviceState::new(),
        }
    }

    pub fn mouse_movement_listener(&self, callback: impl Fn(MousePosition) -> ()) {
        let mut last_pos = (0, 0);

        loop {
            let pos = self.mouse.get_mouse().coords;
            let comparison = self.compare_positions(&last_pos, &pos);
            if comparison.movement() {
                callback(MousePosition { x: pos.0, y: pos.1 });
            }

            last_pos = pos;

            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn compare_positions(&self, point_1: &(i32, i32), point_2: &(i32, i32)) -> MouseMovement {
        MouseMovement {
            x: point_2.0 - point_1.0,
            y: point_2.1 - point_1.1,
        }
    }
}
