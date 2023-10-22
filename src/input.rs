use mouse_rs::{types::Point, Mouse};

pub struct MouseInputReceiver {
    mouse: Mouse,
}

pub struct MouseMovement {
    pub x: i32,
    pub y: i32,
}

impl MouseInputReceiver {
    pub fn new() -> Self {
        MouseInputReceiver {
            mouse: Mouse::new(),
        }
    }

    pub fn mouse_movement_listener(&self, callback: impl Fn(MouseMovement) -> ()) {
        let last_pos: Point = Point { x: 0, y: 0 };

        loop {
            let pos = self
                .mouse
                .get_position()
                .expect("Was unable to get mouse-position. Are you on wayland?");
            let comparison = self.compare_positions(&last_pos, &pos);
            callback(comparison);
        }
    }

    fn compare_positions(&self, point_1: &Point, point_2: &Point) -> MouseMovement {
        MouseMovement {
            x: point_2.x - point_1.x,
            y: point_2.y - point_1.y,
        }
    }
}
