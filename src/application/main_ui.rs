use std::{borrow::BorrowMut, error::Error};

use eframe::Frame;
use egui::{Color32, Context, Vec2};

use crate::{
    application::{backend::server::UsersStatus, QueryMessage},
    cache_utils::{get_cache_value, set_cache_value},
};

use super::Application;

// Possible modifications the user can make to the data using the interface
enum Modification {
    // Update the whole email and redirection entry
    // This also results in insertion if the email is not already in the list
    UpdateEmail {
        email: String,
        redirections: Vec<String>,
    },
    // Remove the whole email
    RemoveEmail(String),
}

// Implementation for the application's main ui
impl Application {
    pub fn draw_main(&mut self, ctx: &Context, _: &mut Frame) -> Result<(), Box<dyn Error>> {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Simple variable to hold the selected server instance's index
            let mut selected_server: usize = get_cache_value("current_server", ui, Some(0));

            let some_server_is_busy = self.servers.iter().any(|x| x.busy());

            // Horizontal widget to show the server selection buttons
            ui.horizontal(|ui| {
                ui.label("Servidores:");
                ui.add_space(10.0);
                ui.separator();

                // Add a scroll so there can be as many servers as needed
                egui::ScrollArea::horizontal()
                    .id_source("servers")
                    .show(ui, |ui| {
                        // Iterate over the servers and add the buttons
                        for (index, server) in self.servers.iter().enumerate() {
                            // Show in green the selected server's button
                            ui.visuals_mut().override_text_color = if selected_server == index {
                                Some(Color32::GREEN)
                            } else {
                                None
                            };

                            // Add the button
                            if ui
                                .small_button(&server.addr)
                                // Tooltip for the button(the server's configuration path)
                                .on_hover_text(&server.config_path)
                                .clicked()
                            {
                                // Handle click:
                                // Set the variable and save it to the cache
                                selected_server = index;
                                set_cache_value("current_server", ui, selected_server);
                            }
                        }
                    });
            });

            // Initialize a list of possible modifications, tipically there will be only one per iteration, but the door is open
            // so there can be composed actions where more than one entry is modified(removed/inserted)
            let mut modifications: Vec<Modification> = vec![];

            // This is scoped so the self.servers borrow is released after the scope is exited
            {
                if self.servers[selected_server].users_status != UsersStatus::Unknown {
                    ui.separator();

                    // A little heading
                    ui.horizontal(|ui| {
                        // Show a button that allows the user to upload the configuration to the server
                        ui.add_enabled_ui(
                            !some_server_is_busy
                                && !self.servers[selected_server]
                                    .users
                                    .iter()
                                    .any(|(_, redirections)| redirections.is_empty()),
                            |ui| {
                                if ui.button("Guardar en el servidor").clicked() {
                                    log::trace!("Saving...");

                                    // Mark the server as busy
                                    self.servers[selected_server].users_status =
                                        UsersStatus::Uploading;

                                    // Send a query to the backend, so it handles the heavy stuffs
                                    let _ = self.tx.send(QueryMessage::UpdateVirtualUsers(
                                        self.servers[selected_server].clone(),
                                    ));
                                }
                            },
                        );

                        // Show a spinner if the backend is still uploading the data to the server
                        if self
                            .servers
                            .iter()
                            .any(|x| x.users_status == UsersStatus::Uploading)
                        {
                            ui.spinner();
                        }
                    });
                }

                // Get the redirections in a variable so it's easyer to type
                let server_redirections = &self.servers[selected_server].users;

                // Show the items in a vertical scroll area, so it's free to grow as needed
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Get the redirection keys and sort them, so it's uniform and consistent among iterations
                    let mut keys: Vec<&String> = server_redirections.keys().collect();
                    keys.sort();

                    // Iterate over the keys
                    keys.into_iter().for_each(|mail| {
                        // Get the redirections for that key
                        let redirections = server_redirections.get(mail).unwrap();

                        // Show the data grouped(This is more appealing)
                        ui.collapsing(mail, |ui| {
                            // Make the group allocate the whole horizontal space, so it's uniform
                            ui.allocate_space(Vec2::new(ui.available_width(), 0.0));
                            // Horizontal widget to hold the email and the delete button
                            ui.horizontal(|ui| {
                                let mut email: String = get_cache_value(
                                    &(mail.to_owned() + "e"),
                                    ui,
                                    Some(mail.to_owned()),
                                );
                                if ui.text_edit_singleline(&mut email).changed() {
                                    set_cache_value(&(mail.to_owned() + "e"), ui, email.clone());
                                }

                                if ui.small_button("Cambiar").clicked() {
                                    modifications.push(Modification::UpdateEmail {
                                        email,
                                        redirections: redirections.clone(),
                                    });
                                    modifications.push(Modification::RemoveEmail(mail.to_owned()));
                                }
                                // ui.add_space(40.0);
                                // Add the delete button
                                if ui.small_button("❌").clicked() {
                                    log::trace!("Removing entry");
                                    modifications.push(Modification::RemoveEmail(mail.to_owned()));
                                }
                            });

                            ui.add_space(10.0);
                            ui.label("Redirecciones");

                            // Iterate over the redirections, adding the entries
                            for redir in redirections {
                                // Horizontal widget to hold the item's label and the delete button
                                ui.horizontal(|ui| {
                                    // Add the delete button
                                    if ui.small_button("❌").clicked() {
                                        // Just push a modification wich eliminates this entry
                                        modifications.push(Modification::UpdateEmail {
                                            email: mail.to_owned(),
                                            // Filter redirections removing this entry
                                            redirections: redirections
                                                .iter()
                                                .filter_map(|entry| {
                                                    if entry != redir {
                                                        Some(entry.to_owned())
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .collect(),
                                        })
                                    }
                                    // Add the email label
                                    ui.label(redir);
                                    ui.add_space(10.0);
                                });
                            }

                            // Get a temporary input string that will hold the text for the text input for that email
                            let mut temp_input: String = get_cache_value(mail, ui, None);

                            ui.separator();
                            // Add a small label
                            ui.small("Añadir redirección");
                            // Then add the text input and a button to add the entry
                            ui.horizontal(|ui| {
                                // Update the cached value only when the user changes the value
                                if (&ui.text_edit_singleline(&mut temp_input)).changed() {
                                    set_cache_value(mail, ui, temp_input.clone());
                                };

                                // Add the 'add' button only when there is valid data
                                ui.add_enabled_ui(!temp_input.trim().is_empty(), |ui| {
                                    if ui.small_button("Añadir").clicked() {
                                        // Clone the redirections
                                        let mut redirection = redirections.clone();

                                        // Add the new entry
                                        redirection.push(temp_input);

                                        // And then push the modification into the list
                                        modifications.push(Modification::UpdateEmail {
                                            email: mail.to_owned(),
                                            redirections: redirection,
                                        });

                                        // Clear the cached input text, so it's cleared and ready to accept further input from the user
                                        set_cache_value(mail, ui, String::new());
                                    }
                                });
                            });
                        });

                        ui.separator();
                    });
                });

                // If the server's data hasn't arrived yet show a spinner and a label indicating so
                if self.servers[selected_server].users_status == UsersStatus::Downloading {
                    ui.heading("Esperando información del servidor...");
                    ui.spinner();
                } else {
                    // Else show an input field so the user can add more redirections
                    ui.label("Añadir redirección");

                    // The horizontal widget that holds the input and the button
                    ui.horizontal(|ui| {
                        // This holds the user input(cached value)
                        let mut email: String = get_cache_value("email", ui, None);

                        // Update the value only if the user changes the text
                        if ui.text_edit_singleline(&mut email).changed() {
                            set_cache_value("email", ui, email.clone());
                        }

                        // Add the 'Add' button
                        ui.add_enabled_ui(!email.trim().is_empty(), |ui| {
                            if ui.small_button("Añadir").clicked() {
                                // Add the email to the list and push the modification
                                modifications.push(Modification::UpdateEmail {
                                    email: email.clone(),
                                    redirections: vec![],
                                });
                                // Clear the cached input text
                                set_cache_value("email", ui, String::new());
                            }
                        });
                    });
                }
            }
            // Now the borrow it's out of scope we can modify self.servers once again

            // Process the modifications
            for modification in modifications {
                match modification {
                    // Email update/insertion modification
                    Modification::UpdateEmail {
                        email,
                        redirections,
                    } => {
                        // Get a mut borrow of the user list
                        let server_redirections = self.servers[selected_server].users.borrow_mut();
                        // If there is already an entry for that email
                        if let Some(redirection) = server_redirections.get_mut(&email) {
                            // just update the content
                            *redirection = redirections;
                        } else {
                            // else insert it into the hash
                            server_redirections.insert(email, redirections);
                        }
                    }

                    // Email deletion modification
                    Modification::RemoveEmail(email) => {
                        // Remove the entry from the hash
                        self.servers[selected_server].users.remove(&email);
                    }
                }
            }
        });

        Ok(())
    }
}
