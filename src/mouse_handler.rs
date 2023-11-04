use {
    crate::{
        display::{Client, DisplayError, DisplayManager},
        gui::GUI,
        input::{MouseMovement, MousePosition},
        protocol::{EventHandler, ProtocolError},
    },
    enigo::{Enigo, MouseControllable},
    std::{error, fmt, sync::Arc},
    tokio::sync::Mutex,
};

type Result<T> = std::result::Result<T, MouseHandlerError>;

#[derive(Debug)]
pub enum MouseHandlerError {
    DisplayError(DisplayError),
    ProtocolError(ProtocolError),
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

pub struct Handler {
    event_handler: Arc<EventHandler>,
    enigo: Enigo,
    display_manager: Arc<Mutex<DisplayManager>>,
    current_position: MousePosition,
    gui: GUI,
}

impl Handler {
    pub fn new(
        event_handler: Arc<EventHandler>,
        display_manager: Arc<Mutex<DisplayManager>>,
    ) -> Handler {
        Handler {
            event_handler,
            enigo: Enigo::new(),
            display_manager,
            current_position: MousePosition { x: 0, y: 0 },
            gui: GUI::new(),
        }
    }

    pub async fn mouse_movement(&mut self, mouse_movement: MouseMovement) -> Result<()> {
        let before_position;
        {
            before_position = self
                .display_manager
                .lock()
                .await
                .get_local_mouse_position(&self.current_position)?;
            self.current_position += mouse_movement;
        }

        match before_position.client {
            Client::IsNetworked(_) => {
                self.apply_current_position().await?;
            }
            Client::IsSelf => {}
        }

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

        match new_position.client {
            Client::IsSelf => {
                self.enigo
                    .mouse_move_to(new_position.mouse_position.x, new_position.mouse_position.y);
                if self.gui.enabled() {
                    self.gui.quit_ui()
                }
            }
            Client::IsNetworked(_) => {
                if !self.gui.enabled() {
                    self.gui.init_ui();
                }
                let size = self.enigo.main_display_size();
                self.enigo.mouse_move_to(size.0, size.1);
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
