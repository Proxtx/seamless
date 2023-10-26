use {
    crate::{communicate::Communicate, input::MouseMovement},
    std::{error, fmt},
};

type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(Debug)]
enum ProtocolError {
    ParserError(String, String),
}

impl error::Error for ProtocolError {}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ProtocolError::ParserError(parser, error) => {
                write!(
                    f,
                    "Parser '{}' for protocol had an exception: {}",
                    parser, error
                )
            }
        }
    }
}

pub trait Event {
    fn serialize(&self) -> String;
    fn parse(text: String) -> Result<Box<Self>>;
    const IDENTIFIER: &'static str;
}

struct MouseMove {
    movement: MouseMovement,
}

impl Event for MouseMove {
    const IDENTIFIER: &'static str = "M";
    fn parse(text: String) -> Result<Box<Self>> {
        Ok(Box::new(MouseMove {
            movement: MouseMovement { x: 0, y: 0 },
        }))
    }
}

pub struct EventHandler {
    communicate: Communicate,
}

impl EventHandler {}
