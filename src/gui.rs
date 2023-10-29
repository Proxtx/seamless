use eframe::egui;
use std::collections::HashMap;

pub struct GUI {}

impl GUI {
    pub fn new() -> Self {
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(320.0, 240.0)),
            ..Default::default()
        };

        eframe::run_native(
            "Seamless",
            options,
            Box::new(|cc| Box::<SeamlessUI>::default()),
        )
        .unwrap();

        GUI {}
    }
}

struct SeamlessUI {
    display_items: HashMap<egui::Vec2, String>,
}

impl Default for SeamlessUI {
    fn default() -> Self {
        Self {
            display_items: HashMap::new(),
        }
    }
}

impl eframe::App for SeamlessUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {}
}
