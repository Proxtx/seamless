use {
    eframe::{
        egui::{self, CursorIcon},
        Frame,
    },
    std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    tokio::sync::mpsc,
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
    gui: GUI,
    receiver: mpsc::UnboundedReceiver<bool>,
}

impl GUIHandler {
    pub fn new() -> (GUIHandler, mpsc::UnboundedSender<bool>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (
            GUIHandler {
                gui: GUI::new(),
                receiver,
            },
            sender,
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
