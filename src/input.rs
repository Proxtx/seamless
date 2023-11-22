use std::str::FromStr;

use {
    crate::{
        display::Client,
        mouse_handler::Handler,
        mouse_handler::Handler as MouseHandler,
        protocol::{EventHandler, ProtocolError},
    },
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

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
}

impl From<&Direction> for String {
    fn from(value: &Direction) -> Self {
        match value {
            Direction::Down => String::from("down"),
            Direction::Up => String::from("up"),
        }
    }
}

impl TryFrom<String> for Direction {
    type Error = ProtocolError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "down" => Ok(Self::Down),
            "up" => Ok(Self::Up),
            _ => Err(ProtocolError::ParserError(
                "Key direction",
                String::from("Key direction could not be parsed"),
            )),
        }
    }
}

#[derive(Debug)]
pub enum Key {
    KeyCode(Keycode),
    MouseButton(usize),
}

impl Key {
    fn prepare_text(prefix: &'static str, text: &mut String) {
        for _ in 0..prefix.len() {
            text.remove(0);
        }
    }
}

impl From<&Key> for String {
    fn from(value: &Key) -> Self {
        match value {
            Key::KeyCode(code) => String::from("K_".to_string() + &code.to_string()),
            Key::MouseButton(index) => String::from("M_".to_string() + &index.to_string()),
        }
    }
}

impl From<Keycode> for Key {
    fn from(value: Keycode) -> Self {
        Key::KeyCode(value)
    }
}

impl From<usize> for Key {
    fn from(value: usize) -> Self {
        Key::MouseButton(value)
    }
}

impl TryFrom<String> for Key {
    type Error = ProtocolError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.starts_with("K_") {
            let mut cut_text = value.clone();
            Key::prepare_text("K_", &mut cut_text);
            return match Keycode::from_str(&cut_text) {
                Ok(v) => Ok(Key::KeyCode(v)),
                Err(e) => Err(ProtocolError::ParserError(
                    "Key Parser",
                    format!("Unable to get key for: {}. Error: {}", cut_text, e),
                )),
            };
        } else if value.starts_with("M_") {
            let mut cut_text = value.clone();
            Key::prepare_text("M_", &mut cut_text);
            return match cut_text.parse::<usize>() {
                Ok(v) => Ok(Key::MouseButton(v)),
                Err(e) => Err(ProtocolError::ParserError(
                    "Key Parser",
                    format!("Unable to get mouse button for: {}. Error: {}", cut_text, e),
                )),
            };
        }

        Err(ProtocolError::ParserError(
            "Key Parser",
            String::from("Key does not start with a known prefix"),
        ))
    }
}

#[derive(Debug)]
pub struct KeyInput {
    pub key: Key,
    pub direction: Direction,
}

impl TryFrom<String> for KeyInput {
    type Error = ProtocolError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut split = value.split("|");
        match (split.next(), split.next()) {
            (Some(dir), Some(key)) => {
                let direction = Direction::try_from(String::from(dir))?;
                let key = Key::try_from(String::from(key))?;
                Ok(KeyInput::new(key, direction))
            }
            _ => {
                return Err(ProtocolError::ParserError(
                    "Key Input Parser",
                    String::from("Unable to parse key inputs. Not enough segments"),
                ));
            }
        }
    }
}

impl From<&KeyInput> for String {
    fn from(value: &KeyInput) -> Self {
        String::from(&value.direction) + "|" + &String::from(&value.key)
    }
}

impl KeyInput {
    pub fn new(key: Key, direction: Direction) -> KeyInput {
        KeyInput { key, direction }
    }
}

pub struct KeyInputReceiver {
    keys: DeviceState,
    event_handler: Arc<EventHandler>,
    mouse_handler: Arc<Mutex<MouseHandler>>,
}

impl KeyInputReceiver {
    pub fn new(event_handler: Arc<EventHandler>, mouse_handler: Arc<Mutex<MouseHandler>>) -> Self {
        KeyInputReceiver {
            keys: DeviceState::new(),
            event_handler,
            mouse_handler,
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
        let event_handler_cls = self.event_handler.clone();
        let mouse_handler_cls = self.mouse_handler.clone();
        let handle_cls = handle.clone();
        let key_down_guard = self.keys.on_key_down(move |key| {
            let key = key.clone();
            let event_handler = event_handler_cls.clone();
            let mouse_handler = mouse_handler_cls.clone();
            handle_cls.clone().spawn(async move {
                let mouse_client_res;
                {
                    let lock = mouse_handler.lock().await;
                    mouse_client_res = lock.get_local_mouse_position().await;
                }
                let mouse_client_position = match mouse_client_res {
                    Err(e) => {
                        return println!(
                        "Unable to read mouse position to transmit keys to the correct client: {}",
                        e
                    )
                    }
                    Ok(v) => v,
                };

                let client_addr = match mouse_client_position.client {
                    Client::IsNetworked(v) => v,
                    Client::IsSelf => {
                        return;
                    }
                };

                match event_handler
                    .secure_communication(
                        client_addr,
                        Box::new(KeyInput::new(Key::from(key), Direction::Down)),
                    )
                    .await
                {
                    Err(e) => {
                        println!("Error sending key: {}", e)
                    }
                    _ => {}
                }
            });
        });

        let event_handler_cls = self.event_handler.clone();
        let mouse_handler_cls = self.mouse_handler.clone();
        let handle_cls = handle.clone();
        let key_up_guard = self.keys.on_key_up(move |key| {
            let key = key.clone();
            let event_handler = event_handler_cls.clone();
            let mouse_handler = mouse_handler_cls.clone();
            handle_cls.clone().spawn(async move {
                let mouse_client_res;
                {
                    let lock = mouse_handler.lock().await;
                    mouse_client_res = lock.get_local_mouse_position().await;
                }
                let mouse_client_position = match mouse_client_res {
                    Err(e) => {
                        return println!(
                        "Unable to read mouse position to transmit keys to the correct client: {}",
                        e
                    )
                    }
                    Ok(v) => v,
                };

                let client_addr = match mouse_client_position.client {
                    Client::IsNetworked(v) => v,
                    Client::IsSelf => {
                        return;
                    }
                };

                match event_handler
                    .secure_communication(
                        client_addr,
                        Box::new(KeyInput::new(Key::from(key), Direction::Up)),
                    )
                    .await
                {
                    Err(e) => {
                        println!("Error sending key: {}", e)
                    }
                    _ => {}
                }
            });
        });

        let event_handler_cls = self.event_handler.clone();
        let mouse_handler_cls = self.mouse_handler.clone();
        let handle_cls = handle.clone();
        let mouse_down_guard = self.keys.on_mouse_down(move |key| {
            let key = key.clone();
            let event_handler = event_handler_cls.clone();
            let mouse_handler = mouse_handler_cls.clone();
            handle_cls.clone().spawn(async move {
                let mouse_client_res;
                {
                    let lock = mouse_handler.lock().await;
                    mouse_client_res = lock.get_local_mouse_position().await;
                }
                let mouse_client_position = match mouse_client_res {
                    Err(e) => {
                        return println!(
                        "Unable to read mouse position to transmit keys to the correct client: {}",
                        e
                    )
                    }
                    Ok(v) => v,
                };

                let client_addr = match mouse_client_position.client {
                    Client::IsNetworked(v) => v,
                    Client::IsSelf => {
                        return;
                    }
                };

                match event_handler
                    .secure_communication(
                        client_addr,
                        Box::new(KeyInput::new(Key::from(key), Direction::Down)),
                    )
                    .await
                {
                    Err(e) => {
                        println!("Error sending key: {}", e)
                    }
                    _ => {}
                }
            });
        });

        let event_handler_cls = self.event_handler.clone();
        let mouse_handler_cls = self.mouse_handler.clone();
        let handle_cls = handle.clone();
        let mouse_up_guard = self.keys.on_mouse_up(move |key| {
            let key = key.clone();
            let event_handler = event_handler_cls.clone();
            let mouse_handler = mouse_handler_cls.clone();
            handle_cls.clone().spawn(async move {
                let mouse_client_res;
                {
                    let lock = mouse_handler.lock().await;
                    mouse_client_res = lock.get_local_mouse_position().await;
                }
                let mouse_client_position = match mouse_client_res {
                    Err(e) => {
                        return println!(
                        "Unable to read mouse position to transmit keys to the correct client: {}",
                        e
                    )
                    }
                    Ok(v) => v,
                };

                let client_addr = match mouse_client_position.client {
                    Client::IsNetworked(v) => v,
                    Client::IsSelf => {
                        return;
                    }
                };

                match event_handler
                    .secure_communication(
                        client_addr,
                        Box::new(KeyInput::new(Key::from(key), Direction::Up)),
                    )
                    .await
                {
                    Err(e) => {
                        println!("Error sending key: {}", e)
                    }
                    _ => {}
                }
            });
        });

        (
            key_down_guard,
            key_up_guard,
            mouse_down_guard,
            mouse_up_guard,
        )
    }
}
