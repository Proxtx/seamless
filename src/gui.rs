use {
    eframe::{
        egui::{self, CursorIcon, ViewportBuilder},
        Frame,
    },
    std::{
        borrow::BorrowMut,
        option,
        process::{Child, Command},
    },
    tokio::sync::mpsc::{self, error::SendError},
};

pub struct GUI {}

impl GUI {
    pub fn new() -> Self {
        let mut options = eframe::NativeOptions::default();
        options.viewport = options
            .viewport
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_inner_size(egui::vec2(320.0, 240.0));
        let ui = SeamlessUI::new();

        eframe::run_native("Seamless", options, Box::new(|_cc| Box::new(ui)))
            .expect("Was unable to create window. Panic! 🚨");

        GUI {}
    }
}

struct SeamlessUI {}

impl SeamlessUI {
    pub fn new() -> Self {
        SeamlessUI {}
    }
}

impl eframe::App for SeamlessUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        ctx.set_cursor_icon(CursorIcon::None);
    }
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
}

pub struct GUIHandler {
    sender: mpsc::UnboundedSender<bool>,
}

impl GUIHandler {
    pub fn quit_ui(&self) -> Result<(), SendError<bool>> {
        self.sender.send(false)?;
        Ok(())
    }

    pub fn init_ui(&self) -> Result<(), SendError<bool>> {
        self.sender.send(true)?;
        Ok(())
    }
}

pub struct GUIProcessManager {
    gui_process: Option<Child>,
    own_path: String,
    receiver: mpsc::UnboundedReceiver<bool>,
}

impl GUIProcessManager {
    pub fn new(own_path: String) -> (Self, GUIHandler) {
        let (sender, receiver) = mpsc::unbounded_channel::<bool>();
        (
            GUIProcessManager {
                gui_process: None,
                own_path,
                receiver,
            },
            GUIHandler { sender },
        )
    }

    fn quit_ui(&mut self) -> Result<(), std::io::Error> {
        match self.gui_process {
            Some(ref mut v) => {
                v.kill()?;
                self.gui_process = None;
            }

            None => {}
        }

        Ok(())
    }

    fn init_ui(&mut self) -> Result<(), std::io::Error> {
        match self.gui_process {
            None => {
                self.gui_process = Some(Command::new(&self.own_path).arg("gui").spawn()?);
            }
            Some(_) => {}
        }
        Ok(())
    }

    pub async fn listen(&mut self) {
        loop {
            match self.receiver.recv().await {
                Some(true) => match self.init_ui() {
                    Err(e) => {
                        println!("Was unable to init ui: {}", e)
                    }
                    _ => {}
                },
                Some(false) => match self.quit_ui() {
                    Err(e) => {
                        println!("Was unable to quit ui: {}", e)
                    }

                    _ => {}
                },
                None => {
                    println!("Received nothing? What how?")
                }
            }
        }
    }
}
