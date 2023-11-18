use {
    crate::{
        display::{Client, DisplayError, DisplayManager},
        gui::GUIHandler,
        input::{MouseMovement, MousePosition},
        protocol::{EventHandler, ProtocolError},
    },
    enigo::{Enigo, MouseControllable},
    std::{error, fmt, sync::Arc},
    tokio::sync::{mpsc, Mutex},
};

type Result<T> = std::result::Result<T, MouseHandlerError>;

#[derive(Debug)]
pub enum MouseHandlerError {
    DisplayError(DisplayError),
    ProtocolError(ProtocolError),
    SendError,
}

impl error::Error for MouseHandlerError {}

impl fmt::Display for MouseHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MouseHandlerError::DisplayError(v) => {
                write!(f, "DisplayError: {}", v)
            }
            MouseHandlerError::ProtocolError(v) => {
                write!(f, "Protocol Error: {}", v)
            }
            MouseHandlerError::SendError => {
                write!(f, "Was unable to send to main thread")
            }
        }
    }
}

impl From<DisplayError> for MouseHandlerError {
    fn from(value: DisplayError) -> Self {
        MouseHandlerError::DisplayError(value)
    }
}

impl From<ProtocolError> for MouseHandlerError {
    fn from(value: ProtocolError) -> Self {
        MouseHandlerError::ProtocolError(value)
    }
}

impl From<mpsc::error::SendError<bool>> for MouseHandlerError {
    fn from(_value: mpsc::error::SendError<bool>) -> Self {
        MouseHandlerError::SendError
    }
}

pub struct Handler {
    event_handler: Arc<EventHandler>,
    enigo: Enigo,
    display_manager: Arc<Mutex<DisplayManager>>,
    current_position: MousePosition,
    gui_handler: Arc<GUIHandler>,
}

impl Handler {
    pub fn new(
        event_handler: Arc<EventHandler>,
        display_manager: Arc<Mutex<DisplayManager>>,
        gui_handler: Arc<GUIHandler>,
    ) -> Handler {
        Handler {
            event_handler,
            enigo: Enigo::new(),
            display_manager,
            current_position: MousePosition { x: 0, y: 0 },
            gui_handler,
        }
    }

    pub async fn mouse_movement(&mut self, mouse_position: MousePosition) -> Result<()> {
        /*let before_position;
        let current_position;
        {
            let lock = self.display_manager.lock().await;
            before_position = lock.get_local_mouse_position(&self.current_position)?;
            self.current_position += &mouse_movement;
            let current_position_res = lock.get_local_mouse_position(&self.current_position);

            drop(lock);

            match current_position_res {
                Ok(v) => {
                    current_position = v;
                }
                Err(e) => match e {
                    DisplayError::InvalidMousePosition => {
                        println!("Do it");
                        self.current_position -= &mouse_movement;
                        self.apply_current_position().await?;
                        return Ok(());
                    }
                    _ => return Err(e.into()),
                },
            }
        }

        match before_position.client {
            Client::IsNetworked(_) => {
                self.apply_current_position().await?;
            }
            Client::IsSelf => {}
        }

        match current_position.client {
            Client::IsNetworked(_) => {
                self.event_handler
                    .emit_event(Box::new(self.current_position.clone()))
                    .await?
            }
            Client::IsSelf => {}
        }

        Ok(())*/

        let new_global_position;
        let is_local: (bool, Option<(i32, i32)>);

        {
            let lock = self.display_manager.lock().await;
            let last_position = lock.get_local_mouse_position(&self.current_position)?;
            match last_position.client {
                Client::IsSelf => {
                    new_global_position = lock.get_global_mouse_position(mouse_position)?;
                    is_local = (true, None);
                }
                Client::IsNetworked(_) => {
                    let display_size = self.enigo.main_display_size();
                    new_global_position = self.current_position.clone()
                        + MouseMovement {
                            x: (display_size.0 / 2 - mouse_position.x),
                            y: (display_size.1 / 2 - mouse_position.y),
                        };
                    is_local = (false, Some(display_size));
                }
            }
        }

        match is_local.0 {
            false => match is_local.1 {
                Some(v) => {
                    self.enigo.mouse_move_to(v.0 / 2, v.1 / 2);
                    self.gui_handler.init_ui()?;
                }
                None => {
                    println!("Local display_size not defined! Why?");
                }
            },

            true => {
                self.gui_handler.quit_ui();
            }
        }

        self.current_position = new_global_position;

        println!("{:?}", self.current_position);

        self.event_handler
            .emit_event(Box::new(self.current_position.clone()))
            .await?;

        Ok(())
    }

    pub async fn apply_current_position(&mut self) -> Result<()> {
        let new_position;

        {
            new_position = self
                .display_manager
                .lock()
                .await
                .get_local_mouse_position(&self.current_position)?;
        }

        println!("{:?}, {:?}", new_position, self.current_position);

        match new_position.client {
            Client::IsSelf => {
                self.enigo
                    .mouse_move_to(new_position.mouse_position.x, new_position.mouse_position.y);
                self.gui_handler.quit_ui();
            }
            Client::IsNetworked(_) => {
                self.gui_handler.init_ui()?;
                let size = self.enigo.main_display_size();
                self.enigo.mouse_move_to(size.0 / 2, size.1 / 2);
            }
        }

        Ok(())
    }

    pub async fn set_current_position(&mut self, current_position: MousePosition) -> Result<()> {
        self.current_position = current_position;
        self.apply_current_position().await?;
        Ok(())
    }
}
