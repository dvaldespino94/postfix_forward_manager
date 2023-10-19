#![windows_subsystem = "windows"]

use eframe::IconData;
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
            icon_data: Some(load_icon(include_bytes!("../icon.png").to_vec())),
            ..Default::default()
        },
        Box::new(|ctx| application::Application::new(ctx)),
    )?;

    log::trace!("Done");

    Ok(())
}

fn load_icon(include_bytes: Vec<u8>) -> eframe::IconData {
    IconData::try_from_png_bytes(&include_bytes).expect("Couldn't load icon from PNG data!")
}
