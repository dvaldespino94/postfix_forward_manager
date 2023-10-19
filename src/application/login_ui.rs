use std::{error::Error, process::exit};

use super::{Application, QueryMessage};

// Implementation for the application's login ui
impl Application {
    pub fn draw_login(&mut self, ui: &mut egui::Ui) -> Result<(), Box<dyn Error>> {
        ui.heading("Login");

        // User name text input
        ui.horizontal(|ui| {
            ui.label("Usuario:");
            ui.text_edit_singleline(&mut self.username);
        });

        // Password text input
        ui.horizontal(|ui| {
            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
        });

        // Root password text input
        ui.horizontal(|ui| {
            ui.label("Password root:");
            ui.add(egui::TextEdit::singleline(&mut self.root_password).password(true));
        });

        ui.add_space(10.0);

        // The buttons
        ui.horizontal(|ui| {
            // Only enable the login button if the form data is valid
            ui.add_enabled_ui(self.login_form_is_valid(), |ui| {
                if self.servers.iter().any(|x| x.busy) {
                    // If there is a login in progress show a spinner or checkmark for each instance

                    ui.horizontal(|ui| {
                        for server in self.servers.iter() {
                            if server.busy {
                                ui.spinner();
                            } else {
                                ui.label("✅");
                            }
                        }
                    });
                } else {
                    // Add the login button
                    if ui.button("Ok").clicked() {
                        for server in self.servers.iter_mut() {
                            server.busy = true;
                        }

                        // Send a query to the backend
                        let _ = self.tx.send(QueryMessage::Authenticate {
                            username: self.username.clone(),
                            password: self.password.clone(),
                            servers: self.servers.clone(),
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

        Ok(())
    }
}
