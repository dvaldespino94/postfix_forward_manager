use eframe::App;
use egui::{Color32, Vec2};

pub struct ErrorApplication {
    pub message: String,
}

impl ErrorApplication {
    pub fn new(message: String) -> Box<dyn App> {
        Box::new(Self { message })
    }
}

impl App for ErrorApplication {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(Color32::RED);
            ui.heading("There was an error starting the application");
            ui.label(&self.message);
            if ui.button("Close").clicked() {
                frame.close();
            }

            frame.set_window_size(Vec2::new(200.0, 120.0));
        });
    }
}
