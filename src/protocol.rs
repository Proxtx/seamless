use {
    crate::{
        communicate::{Communicate, CommunicateError},
        display::ClientDisplays,
        input::MouseMovement,
    },
    std::{error, fmt, sync::Arc},
};

type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(Debug)]
pub enum ProtocolError {
    ParserError(&'static str, String),
    ParseError,
    CommunicateError(CommunicateError),
    SerdeSerializationError(serde_json::error::Error),
}

impl error::Error for ProtocolError {}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::ParserError(parser, error) => {
                write!(
                    f,
                    "Parser '{}' for protocol had an exception: {}",
                    parser, error
                )
            }
            ProtocolError::ParseError => {
                write!(f, "No matching parser was found!")
            }
            ProtocolError::CommunicateError(error) => {
                write!(f, "Communication Error: {}", error)
            }
            ProtocolError::SerdeSerializationError(e) => {
                write!(f, "Serde serialization error: {}", e)
            }
        }
    }
}

impl From<CommunicateError> for ProtocolError {
    fn from(value: CommunicateError) -> Self {
        ProtocolError::CommunicateError(value)
    }
}

impl From<serde_json::error::Error> for ProtocolError {
    fn from(value: serde_json::error::Error) -> Self {
        ProtocolError::SerdeSerializationError(value)
    }
}

pub trait Event
where
    Self: Send + Sync,
{
    fn serialize(&self) -> Result<String>;
}
impl Event for MouseMovement {
    fn serialize(&self) -> Result<String> {
        Ok(format!("M{}|{}", self.x, self.y))
    }
}

impl Event for ClientDisplays {
    fn serialize(&self) -> Result<String> {
        Ok(format!("D{}", serde_json::to_string(self)?))
    }
}

struct MouseMoveParser {}

impl MouseMoveParser {
    fn parse(&self, text: String) -> Result<MouseMovement> {
        let mut split = text.split("|");
        let x: i32;
        let y: i32;
        match (split.next(), split.next()) {
            (Some(x_t), Some(y_t)) => match (x_t.parse::<i32>(), y_t.parse::<i32>()) {
                (Ok(x_p), Ok(y_p)) => {
                    x = x_p;
                    y = y_p;
                }
                _ => {
                    return Err(ProtocolError::ParserError(
                        "MouseMove",
                        String::from("Number parsing"),
                    ));
                }
            },
            _ => {
                return Err(ProtocolError::ParserError(
                    "MouseMove",
                    String::from("Number amount"),
                ));
            }
        };

        Ok(MouseMovement { x, y })
    }

    fn get_prefix(&self) -> &'static str {
        "M"
    }
}

struct ClientDisplayParser {}

impl ClientDisplayParser {
    fn parse(&self, text: String) -> Result<ClientDisplays> {
        Ok(serde_json::from_str::<ClientDisplays>(&text)?)
    }

    fn get_prefix(&self) -> &'static str {
        "D"
    }
}

pub struct EventHandler {
    communicate: Arc<Communicate>,
    parser: Arc<MainParser>,
}

impl EventHandler {
    pub fn new(communicate: Arc<Communicate>) -> Self {
        EventHandler {
            communicate,
            parser: Arc::new(MainParser::new()),
        }
    }

    pub async fn event_listener<T>(&self, handler: T)
    where
        T: Fn(Events) + Send + Sync + 'static,
    {
        let communicate = self.communicate.clone();
        let parser = self.parser.clone();
        communicate
            .receive(|msg, _src| {
                match parser.parse(msg.to_string()) {
                    Ok(v) => {
                        handler(v);
                    }
                    Err(e) => {
                        println!("Error handling received udp package: {}", e)
                    }
                };
            })
            .await;
    }

    pub async fn emit_event(&self, event: Box<dyn Event>) -> Result<()> {
        Ok(self.communicate.send(event.serialize()?).await?)
    }
}

pub enum Events {
    MouseMovement(MouseMovement),
    ClientDisplays(ClientDisplays),
}

pub struct MainParser {
    mouse_movement_parser: MouseMoveParser,
    client_displays_parser: ClientDisplayParser,
}

impl MainParser {
    pub fn new() -> Self {
        MainParser {
            mouse_movement_parser: MouseMoveParser {},
            client_displays_parser: ClientDisplayParser {},
        }
    }

    fn parse(&self, text: String) -> Result<Events> {
        return if text.starts_with(self.mouse_movement_parser.get_prefix()) {
            let mut text = text.clone();
            self.prepare_text(self.mouse_movement_parser.get_prefix(), &mut text);
            Ok(Events::MouseMovement(
                self.mouse_movement_parser.parse(text)?,
            ))
        } else if (text.starts_with(self.client_displays_parser.get_prefix())) {
            let mut text = text.clone();
            self.prepare_text(self.client_displays_parser.get_prefix(), &mut text);
            Ok(Events::ClientDisplays(
                self.client_displays_parser.parse(text)?,
            ))
        } else {
            Err(ProtocolError::ParseError)
        };
    }

    fn prepare_text(&self, prefix: &'static str, text: &mut String) {
        for _ in 0..prefix.len() {
            text.remove(0);
        }
    }
}
