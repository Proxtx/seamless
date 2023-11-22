use {
    crate::input::{Direction, Key as InputKey, KeyInput},
    device_query::keymap::Keycode,
    enigo::{keycodes::Key, Enigo, KeyboardControllable, MouseButton, MouseControllable},
    std::fmt,
};

type Result<T> = std::result::Result<T, KeyError>;

#[derive(Debug)]
pub enum KeyError {
    TransformationError,
}

impl std::fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyError::TransformationError => {
                write!(
                    f,
                    "Was unable to transform a device_query key into an enigo key."
                )
            }
        }
    }
}

impl std::error::Error for KeyError {}

pub struct Handler {
    enigo: Enigo,
}

impl Handler {
    pub fn new() -> Handler {
        Handler {
            enigo: Enigo::new(),
        }
    }

    pub fn key_input(&mut self, key_input: KeyInput) -> Result<()> {
        match key_input.key {
            InputKey::KeyCode(keycode) => {
                let key = device_query_keycode_to_enigo_key(&keycode)?;
                match key_input.direction {
                    Direction::Down => {
                        self.enigo.key_down(key);
                    }
                    Direction::Up => {
                        self.enigo.key_up(key);
                    }
                };
            }
            InputKey::MouseButton(button) => match key_input.direction {
                Direction::Down => {
                    self.enigo
                        .mouse_down(mouse_button_to_enigo_mouse_button(button)?);
                }
                Direction::Up => self
                    .enigo
                    .mouse_up(mouse_button_to_enigo_mouse_button(button)?),
            },
        }
        Ok(())
    }
}

fn mouse_button_to_enigo_mouse_button(mouse_button: usize) -> Result<MouseButton> {
    match mouse_button {
        1 => Ok(MouseButton::Left),
        2 => Ok(MouseButton::Right),
        3 => Ok(MouseButton::Middle),
        4 => Ok(MouseButton::Back),
        5 => Ok(MouseButton::Forward),
        _ => Err(KeyError::TransformationError),
    }
}

fn device_query_keycode_to_enigo_key(key: &Keycode) -> Result<Key> {
    match key {
        Keycode::A => Ok(Key::A),
        Keycode::B => Ok(Key::B),
        Keycode::C => Ok(Key::C),
        Keycode::D => Ok(Key::D),
        Keycode::E => Ok(Key::E),
        Keycode::F => Ok(Key::F),
        Keycode::G => Ok(Key::G),
        Keycode::H => Ok(Key::H),
        Keycode::I => Ok(Key::I),
        Keycode::J => Ok(Key::J),
        Keycode::K => Ok(Key::K),
        Keycode::L => Ok(Key::L),
        Keycode::M => Ok(Key::M),
        Keycode::N => Ok(Key::N),
        Keycode::O => Ok(Key::O),
        Keycode::P => Ok(Key::P),
        Keycode::Q => Ok(Key::Q),
        Keycode::R => Ok(Key::R),
        Keycode::S => Ok(Key::S),
        Keycode::T => Ok(Key::T),
        Keycode::U => Ok(Key::U),
        Keycode::V => Ok(Key::V),
        Keycode::W => Ok(Key::W),
        Keycode::X => Ok(Key::X),
        Keycode::Y => Ok(Key::Y),
        Keycode::Z => Ok(Key::Z),
        Keycode::Key0 => Ok(Key::Num0),
        Keycode::Key1 => Ok(Key::Num1),
        Keycode::Key2 => Ok(Key::Num2),
        Keycode::Key3 => Ok(Key::Num3),
        Keycode::Key4 => Ok(Key::Num4),
        Keycode::Key5 => Ok(Key::Num5),
        Keycode::Key6 => Ok(Key::Num6),
        Keycode::Key7 => Ok(Key::Num7),
        Keycode::Key8 => Ok(Key::Num8),
        Keycode::Key9 => Ok(Key::Num9),
        Keycode::F1 => Ok(Key::F1),
        Keycode::F2 => Ok(Key::F2),
        Keycode::F3 => Ok(Key::F3),
        Keycode::F4 => Ok(Key::F4),
        Keycode::F5 => Ok(Key::F5),
        Keycode::F6 => Ok(Key::F6),
        Keycode::F7 => Ok(Key::F7),
        Keycode::F8 => Ok(Key::F8),
        Keycode::F9 => Ok(Key::F9),
        Keycode::F10 => Ok(Key::F10),
        Keycode::F11 => Ok(Key::F11),
        Keycode::F12 => Ok(Key::F12),
        Keycode::LControl => Ok(Key::LControl),
        Keycode::RControl => Ok(Key::RControl),
        Keycode::Space => Ok(Key::Space),
        Keycode::Backspace => Ok(Key::Backspace),
        Keycode::Meta => Ok(Key::Meta),
        Keycode::Escape => Ok(Key::Escape),
        Keycode::Enter => Ok(Key::Return),
        Keycode::Up => Ok(Key::UpArrow),
        Keycode::Down => Ok(Key::DownArrow),
        Keycode::Left => Ok(Key::LeftArrow),
        Keycode::Right => Ok(Key::RightArrow),
        Keycode::LAlt => Ok(Key::Alt),
        Keycode::LShift => Ok(Key::LShift),
        Keycode::RShift => Ok(Key::RShift),
        _ => Err(KeyError::TransformationError),
    }
}
