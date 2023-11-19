use {
    crate::{
        display::{Client, ClientMousePosition, DisplayError, DisplayManager, Edge},
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
    OwnDisplayError,
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
            MouseHandlerError::OwnDisplayError => {
                write!(f, "Was unable to find own display")
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
        let last_local_position: Option<ClientMousePosition>;

        let mut new_global_position;
        let new_local_position_res;

        {
            let lock = self.display_manager.lock().await;
            let last_position = lock.get_local_mouse_position(&self.current_position);
            match last_position {
                Ok(last_position) => {
                    match last_position.client {
                        Client::IsSelf => {
                            new_global_position = lock.get_global_mouse_position(mouse_position)?;
                        }
                        Client::IsNetworked(_) => {
                            let display_size = self.enigo.main_display_size();
                            new_global_position = self.current_position.clone()
                                + MouseMovement {
                                    x: (mouse_position.x - display_size.0 / 2) * 2,
                                    y: (mouse_position.y - display_size.1 / 2) * 2,
                                };
                        }
                    }
                    last_local_position = Some(last_position);
                }
                Err(_e) => {
                    new_global_position = MousePosition { x: 0, y: 0 };
                    last_local_position = None;
                }
            }

            let computed_local_position_res = lock.get_local_mouse_position(&new_global_position);
            new_local_position_res = match &computed_local_position_res {
                Ok(v) => match &v.client {
                    Client::IsSelf => {
                        let own_index = match lock.get_own_client_displays_index() {
                            Some(v) => v,
                            None => {
                                return Err(MouseHandlerError::OwnDisplayError);
                            }
                        };

                        match lock.is_on_edge(&v.mouse_position, own_index)? {
                            Some(Edge::Left) => {
                                new_global_position.x -= 5;
                                lock.get_local_mouse_position(&new_global_position)
                            }
                            Some(Edge::Right) => {
                                new_global_position.x += 5;
                                lock.get_local_mouse_position(&new_global_position)
                            }
                            None => computed_local_position_res,
                        }
                    }
                    Client::IsNetworked(_v) => computed_local_position_res,
                },
                Err(_e) => computed_local_position_res,
            }
        }

        match new_local_position_res {
            Ok(new_local_position) => {
                match new_local_position.client {
                    Client::IsNetworked(_) => {
                        self.center_mouse_init_gui()?;
                    }

                    Client::IsSelf => {
                        self.gui_handler.quit_ui()?;
                    }
                }
                self.current_position = new_global_position;
            }
            Err(_e) => match last_local_position {
                Some(v) => match v.client {
                    Client::IsNetworked(_) => {
                        self.center_mouse_init_gui()?;
                    }
                    Client::IsSelf => {}
                },
                None => {}
            },
        }

        self.event_handler
            .emit_event(Box::new(self.current_position.clone()))
            .await?;

        Ok(())
    }

    fn center_mouse_init_gui(&mut self) -> Result<()> {
        let display_size = self.enigo.main_display_size();
        self.enigo
            .mouse_move_to(display_size.0 / 2, display_size.1 / 2);
        self.gui_handler.init_ui()?;
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

        match new_position.client {
            Client::IsSelf => {
                self.enigo
                    .mouse_move_to(new_position.mouse_position.x, new_position.mouse_position.y);
                self.gui_handler.quit_ui()?;
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
