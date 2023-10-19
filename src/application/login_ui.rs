use std::{error::Error, process::exit};

use egui::{Context, Vec2};

use super::{backend::server::AuthStatus, Application, QueryMessage};

// Implementation for the application's login ui
impl Application {
    pub fn draw_login(
        &mut self,
        ctx: &Context,
        frame: &mut eframe::Frame,
    ) -> Result<(), Box<dyn Error>> {
        // Resize the window for the login view
        frame.set_window_size(Vec2::new(400.0, 135.0));

        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.heading("Login");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_width = ui.available_width();
            // User name text input
            ui.horizontal(|ui| {
                let label_width = ui.label("Usuario:").rect.width();
                ui.add(
                    egui::TextEdit::singleline(&mut self.username)
                        .desired_width(available_width - label_width),
                );
            });

            // Password text input
            ui.horizontal(|ui| {
                let label_width = ui.label("Password:").rect.width();
                ui.add(
                    egui::TextEdit::singleline(&mut self.password)
                        .desired_width(available_width - label_width)
                        .password(true),
                );
            });

            // Root password text input
            ui.horizontal(|ui| {
                let label_width = ui.label("Root Password:").rect.width();
                ui.add(
                    egui::TextEdit::singleline(&mut self.root_password)
                        .desired_width(available_width - label_width)
                        .password(true),
                );
            });

            ui.add_space(10.0);

            // The buttons
            ui.horizontal(|ui| {
                // Only enable the login button if the form data is valid
                ui.add_enabled_ui(self.login_form_is_valid(), |ui| {
                    if self.servers.iter().any(|x| x.busy()) {
                        // If there is a login in progress show a spinner or checkmark for each instance

                        ui.horizontal(|ui| {
                            for server in self.servers.iter() {
                                match server.auth_status {
                                    AuthStatus::Unknown => {
                                        ui.label("❓");
                                    }
                                    AuthStatus::Failed => {
                                        ui.label("❌");
                                    }
                                    AuthStatus::Authenticated => {
                                        ui.label("✅");
                                    }
                                    AuthStatus::InProgress => {
                                        ui.spinner();
                                    }
                                };
                            }
                        });
                    } else {
                        // Add the login button
                        if ui.button("Ok").clicked() {
                            for server in self.servers.iter_mut() {
                                if server.auth_status == AuthStatus::Authenticated {
                                    continue;
                                }

                                server.auth_status = AuthStatus::InProgress;
                            }

                            // Send a query to the backend
                            let _ = self.tx.send(QueryMessage::Authenticate {
                                username: self.username.clone(),
                                password: self.password.clone(),
                                servers: self
                                    .servers
                                    .iter()
                                    .filter_map(|x| {
                                        if x.auth_status == AuthStatus::Authenticated {
                                            None
                                        } else {
                                            Some(x.clone())
                                        }
                                    })
                                    .collect(),
                                root_password: self.root_password.clone(),
                            });
                        }
                    }
                });

                // The simplest quit button
                if ui.button("Cancel").clicked() {
                    exit(0);
                }
            });
        });

        Ok(())
    }
}
