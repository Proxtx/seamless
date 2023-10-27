use eframe::egui;

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

struct SeamlessUI {}

impl Default for SeamlessUI {
    fn default() -> Self {
        Self {}
    }
}

impl eframe::App for SeamlessUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.ui.button("Ok");
        });
    }
}
