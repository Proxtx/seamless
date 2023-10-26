use {
    crate::{
        communicate::{Communicate, CommunicateError},
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
        }
    }
}

impl From<CommunicateError> for ProtocolError {
    fn from(value: CommunicateError) -> Self {
        ProtocolError::CommunicateError(value)
    }
}

pub trait Parser
where
    Self: Sync,
{
    fn parse(&self, text: String) -> Result<Box<dyn Event>>;
    fn get_prefix(&self) -> &'static str;
}

pub trait Event {
    fn serialize(&self) -> String;
}
impl Event for MouseMovement {
    fn serialize(&self) -> String {
        format!("M{}|{}", self.x, self.y)
    }
}

struct MouseMoveParser {}

impl Parser for MouseMoveParser {
    fn parse(&self, text: String) -> Result<Box<dyn Event>> {
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

        Ok(Box::new(MouseMovement { x, y }))
    }

    fn get_prefix(&self) -> &'static str {
        "M"
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

    pub fn event_listener<T>(&self, handler: T)
    where
        T: Fn(Box<dyn Event>) + Send + Sync + 'static,
    {
        let communicate = self.communicate.clone();
        let parser = self.parser.clone();
        tokio::spawn(async move {
            communicate
                .receive(|msg, src| {
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
        });
    }

    pub async fn emit_event(&self, event: Box<dyn Event>) -> Result<usize> {
        Ok(self.communicate.send(event.serialize()).await?)
    }
}

pub struct MainParser {
    parsers: Vec<Box<dyn Parser + Send>>,
}

impl MainParser {
    pub fn new() -> Self {
        MainParser {
            parsers: vec![Box::new(MouseMoveParser {})],
        }
    }

    fn parse(&self, text: String) -> Result<Box<dyn Event>> {
        for parser in self.parsers.iter() {
            match self.try_parse(&text, parser) {
                Some(v) => {
                    return v;
                }
                None => {}
            }
        }

        Err(ProtocolError::ParseError)
    }

    fn try_parse(
        &self,
        text: &String,
        parser: &Box<dyn Parser + Send>,
    ) -> Option<Result<Box<dyn Event>>> {
        if !text.starts_with("M") {
            return None;
        }
        Some(parser.parse(text.clone()))
    }
}
