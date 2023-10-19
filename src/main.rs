use egui::Vec2;
use std::error::Error;
mod application;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    log::trace!("Starting");

    eframe::run_native(
        "Email Forwarding Manager",
        eframe::NativeOptions {
            resizable: true,
            initial_window_size: Some(Vec2::new(400.0, 500.0)),
            ..Default::default()
        },
        Box::new(|_| Box::new(application::Application::new())),
    )?;

    log::trace!("Done");

    Ok(())
}
