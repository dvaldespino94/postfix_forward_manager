#![windows_subsystem = "windows"]

use eframe::IconData;
use egui::Vec2;
use std::error::Error;
mod application;
mod cache_utils;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    log::trace!("Starting");

    eframe::run_native(
        "Email Forwarding Manager",
        eframe::NativeOptions {
            resizable: true,
            initial_window_size: Some(Vec2::new(400.0, 500.0)),
            icon_data: Some(load_icon(include_bytes!("../icon.png").to_vec())),
            ..Default::default()
        },
        Box::new(|ctx| Box::new(application::Application::new(ctx))),
    )?;

    log::trace!("Done");

    Ok(())
}

fn load_icon(include_bytes: Vec<u8>) -> eframe::IconData {
    IconData::try_from_png_bytes(&include_bytes).expect("Couldn't load icon from PNG data!")
}
