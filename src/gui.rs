use {
    eframe::{
        egui::{self, CursorIcon},
        Frame,
    },
    std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    tokio::sync::mpsc::{self, error::SendError},
};

pub struct GUI {
    quit: Arc<AtomicBool>,
}

impl GUI {
    pub fn new() -> Self {
        let instance = GUI {
            quit: Arc::new(AtomicBool::new(true)),
        };

        instance
    }

    pub fn init_ui(&self) {
        self.quit.store(false, Ordering::Relaxed);

        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(320.0, 240.0)),
            always_on_top: true,
            centered: true,
            transparent: true,
            decorated: false,
            ..Default::default()
        };
        let ui = SeamlessUI::new(self.quit.clone());

        eframe::run_native("Seamless", options, Box::new(|_cc| Box::new(ui)))
            .expect("Was unable to create window. Panic! 🚨");
    }

    pub fn quit_ui(&self) {
        self.quit.store(true, Ordering::Relaxed)
    }

    pub fn enabled(&self) -> bool {
        !self.quit.load(Ordering::Relaxed)
    }
}

struct SeamlessUI {
    pub quit: Arc<AtomicBool>,
}

impl SeamlessUI {
    pub fn new(quit_bool: Arc<AtomicBool>) -> Self {
        SeamlessUI { quit: quit_bool }
    }
}

impl eframe::App for SeamlessUI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        ctx.set_cursor_icon(CursorIcon::None);
        if self.quit.load(Ordering::Relaxed) {
            frame.close()
        }
    }
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
}

pub struct GUIHandler {
    sender: mpsc::UnboundedSender<bool>,
    end_gui: Arc<AtomicBool>,
}

impl GUIHandler {
    pub fn quit_ui(&self) {
        self.end_gui.store(true, Ordering::Relaxed)
    }

    pub fn init_ui(&self) -> Result<(), SendError<bool>> {
        if !self.end_gui.load(Ordering::Relaxed) {
            self.sender.send(true)?;
        }

        Ok(())
    }
}

pub struct GUIStarter {
    gui: GUI,
    receiver: mpsc::UnboundedReceiver<bool>,
}

impl GUIStarter {
    pub fn new() -> (GUIStarter, GUIHandler) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let gui = GUI::new();
        (
            GUIStarter {
                gui: GUI::new(),
                receiver,
            },
            GUIHandler {
                sender,
                end_gui: gui.quit,
            },
        )
    }

    pub async fn start(&mut self) {
        loop {
            let msg = self.receiver.recv().await;
            match msg {
                None => {
                    println!("Received an empty message on gui channel? Not expected")
                }
                Some(v) => match (v, self.gui.enabled()) {
                    (true, false) => {
                        self.gui.init_ui();
                    }
                    (false, true) => {
                        self.gui.quit_ui();
                    }
                    _ => {}
                },
            }
        }
    }
}
