use std::{
    collections::HashMap,
    ops::ControlFlow,
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use eframe::{App, CreationContext};
use egui::{include_image, TextureOptions, Vec2};
use figment::{
    providers::{Format, Toml},
    Figment,
};

use self::backend::{
    backend_loop,
    configuration::Configuration,
    messages::{QueryMessage, ResponseMessage},
    server::Server,
    sshwrapper::SSHWrapper,
};

mod backend;
mod login_ui;
mod main_ui;

enum Screen {
    Login,
    Main,
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Login
    }
}

// Application struct
pub struct Application {
    // TX Channel
    screen: Screen,
    // RX Channel
    username: String,
    // Login username
    password: String,
    // Login password
    root_password: String,
    // Root Password
    tx: Sender<QueryMessage>,
    // Current Screen
    rx: Receiver<ResponseMessage>,
    // This field also must die soon
    servers: Vec<Server>,
}

impl App for Application {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Spawn a thread that forces to update the ui every 200ms
        ctx.memory_mut(|mem| {
            let already_started: &mut bool = mem
                .data
                .get_temp_mut_or_default("already_instantiated".into());
            if !*already_started {
                log::trace!("Spawning extra update thread");

                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    let ctx_clone = ctx.clone();
                    loop {
                        std::thread::sleep(Duration::from_millis(200));
                        ctx_clone.request_repaint();
                    }
                });
                *already_started = true;
            }
        });

        // Receive and process messages from the backend (different thread)
        if let Ok(msg) = self.rx.try_recv() {
            match msg {
                // Authentication related messages
                ResponseMessage::AuthenticationResult {
                    server,
                    success,
                    error,
                } => {
                    if success {
                        for owned_server in self.servers.iter_mut() {
                            if owned_server == &server {
                                owned_server.busy = false;
                                break;
                            }
                        }

                        log::trace!("Authentication result for server {server}: {success}");
                        if !self.servers.iter().any(|server| server.busy) {
                            // Resize window
                            frame.set_window_size(Vec2::new(400.0, 500.0));

                            // Switch to main view
                            self.screen = Screen::Main;
                        }
                        // Query virtual users for this server
                        let _ = self.tx.send(QueryMessage::QueryVirtualUsers(server));
                    } else {
                        //TODO: Show authentication failed message
                        log::error!(
                            "Authentication failed for server {server}: '{}'",
                            error.unwrap_or_default()
                        );
                    }
                }
                // Handle received virtual users hash
                ResponseMessage::GotVirtualUsers { server, users } => {
                    log::trace!("Got virtual users from server {server}: {users:#?}");

                    // Match the received server instance with the server instances owned by the application
                    for owned_server in self.servers.iter_mut() {
                        if *owned_server == server {
                            // Raise the flag
                            owned_server.received_redirections = true;
                            // Assign the users
                            owned_server.users = users;
                            break;
                        }
                    }
                }
                // Handle the case when the query fails
                ResponseMessage::QueryVirtualUsersResult { server, error } => {
                    //TODO: Show error info
                    log::error!("Couldn't upload configuration to server {server}: {error}");
                }
                // Handle the result of server configuration uploads
                ResponseMessage::ServerUploadResult { error, server } => {
                    for owned_server in self.servers.iter_mut() {
                        if owned_server == &server {
                            owned_server.busy = false;
                            break;
                        }
                    }

                    if let Some(error) = error {
                        log::error!("Error uploading data to server {server}: {error}");
                    } else {
                        log::trace!("Configuration updated successfully for server {server}");
                    }
                }
            }
        }

        // Draw the actual stuffs
        egui::CentralPanel::default().show(ctx, |ui| match self.screen {
            Screen::Login => {
                // Resize the window for the login view
                frame.set_window_size(Vec2::new(300.0, 130.0));

                // Draw the login view
                let _ = self.draw_login(ui);
            }
            Screen::Main => {
                // Draw the main view
                let _ = self.draw_main(ui);
            }
        });
    }
}

impl Application {
    // Create a new instance of the application
    pub fn new(ctx: &CreationContext) -> Self {
        // Load configuration from TOML
        let config: Configuration = Figment::new()
            .merge(Toml::file("config.toml"))
            .extract()
            .unwrap_or_default();

        // Create the channels to communicate with the backend thread

        // Frontend -> Backend
        let (frontend_tx, backend_rx) = channel::<QueryMessage>();
        // Backend -> Frontend
        let (backend_tx, frontend_rx) = channel::<ResponseMessage>();

        // Launch the backend in a different thread
        std::thread::spawn(move || {
            // Create the session hash here to make it persistent between loop iterations
            let mut ssh_sessions: HashMap<String, SSHWrapper> = Default::default();

            // Launch the backend loop
            loop {
                // The loop must be able to stop itself from within, so it returns a ControlFlow
                if let ControlFlow::Break(_) =
                    backend_loop(&backend_rx, &backend_tx, &mut ssh_sessions)
                {
                    return;
                }
            }
        });

        // Return the initialized Application instance
        Self {
            // TX Channel
            tx: frontend_tx,
            // RX Channel
            rx: frontend_rx,
            // Login username
            username: config.username,
            // Login password
            password: String::from(""),
            // Root Password
            root_password: String::from(""),
            // Current Screen
            screen: Default::default(),
            // Server list(loaded from configuration)
            servers: config.servers,
        }
    }

    // Get whether the current data is valid to allow the user click the Ok button
    fn login_form_is_valid(&mut self) -> bool {
        !(self.password.trim().is_empty()
            || self.root_password.trim().is_empty()
            || self.username.trim().is_empty())
    }
}
