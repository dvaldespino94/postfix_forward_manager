use std::{
    collections::HashMap,
    ops::ControlFlow,
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use eframe::{App, CreationContext};
use egui::{Vec2, WidgetText};

use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use figment::{
    providers::{Format, Toml},
    Figment,
};

use crate::application::{
    backend::server::{AuthStatus, UsersStatus},
    errorapplication::ErrorApplication,
};

use self::backend::{
    backend_loop,
    configuration::Configuration,
    messages::{QueryMessage, ResponseMessage},
    server::Server,
    sshwrapper::SSHWrapper,
};

mod backend;
mod errorapplication;
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
    // Toasts 🍞
    toasts: Toasts,
}

impl App for Application {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.toasts.show(ctx);

        // Receive and process messages from the backend (different thread)
        if let Ok(msg) = self.rx.try_recv() {
            match msg {
                // Authentication related messages
                ResponseMessage::AuthenticationResult {
                    server,
                    success,
                    error,
                } => {
                    self.get_server(&server).map(|s| {
                        log::trace!("Setting server to not busy");
                        s.auth_status = success.into();
                    });

                    if success {
                        log::trace!("Authentication result for server {server}: {success}");
                        if self
                            .servers
                            .iter()
                            .all(|server| server.auth_status == AuthStatus::Authenticated)
                        {
                            // Resize window
                            frame.set_window_size(Vec2::new(400.0, 500.0));

                            // Switch to main view
                            self.screen = Screen::Main;
                        }

                        // Query virtual users for this server
                        let _ = self
                            .tx
                            .send(QueryMessage::QueryVirtualUsers(server.clone()));
                        self.get_server(&server).map(|s| {
                            log::trace!("Setting server to busy again");
                            s.users_status = UsersStatus::Downloading;
                        });
                    } else {
                        self.show_notification(
                            format!("Authentication failed\nfor server {server}").into(),
                            ToastKind::Error,
                        );
                        // self.toasts
                        //     .error(format!("Authentication failed for server {server}"))
                        //     .set_closable(true)
                        //     .set_duration(Some(Duration::from_secs(3)));
                        log::error!(
                            "Authentication failed for server {server}: '{}'",
                            error.unwrap_or_default()
                        );
                    }
                }
                // Handle received virtual users hash
                ResponseMessage::GotVirtualUsers { server, users } => {
                    log::trace!("Got virtual users\nfrom server {server}: {users:#?}");

                    // Match the received server instance with the server instances owned by the application
                    self.get_server(&server).map(|s| {
                        s.users = users;
                        s.users_status = UsersStatus::Idle;
                    });
                }
                // Handle the case when the query fails
                ResponseMessage::QueryVirtualUsersResult { server, error } => {
                    self.get_server(&server).map(|s| {
                        s.users_status = UsersStatus::Unknown;
                    });

                    self.show_notification(
                        format!("Couldn't upload configuration\nto server {server}: {error}")
                            .into(),
                        ToastKind::Error,
                    );

                    log::error!("Couldn't upload configuration to server {server}: {error}");
                }
                // Handle the result of server configuration uploads
                ResponseMessage::ServerUploadResult { error, server } => {
                    self.get_server(&server).map(|s| {
                        s.users_status = UsersStatus::Idle;
                    });

                    if let Some(error) = error {
                        log::error!("Error uploading data to server {server}: {error}");

                        self.show_notification(
                            format!("Error uploading data to\nserver {server}: {error}").into(),
                            ToastKind::Error,
                        );
                    } else {
                        log::trace!("Configuration updated successfully for server {server}");
                    }
                }
            }
        }

        // Draw the actual stuffs
        match self.screen {
            Screen::Login => {
                // Draw the login view
                let _ = self.draw_login(&ctx, frame);
            }
            Screen::Main => {
                // Draw the main view
                let _ = self.draw_main(&ctx, frame);
            }
        }
    }
}

impl Application {
    // Create a new instance of the application
    pub fn new(ctx: &CreationContext) -> Box<dyn App> {
        // Load configuration from TOML
        let config: Configuration = match Figment::new().merge(Toml::file("config.toml")).extract()
        {
            Ok(config) => config,
            Err(error) => {
                log::error!("Error loading configuration: {error:#?}");

                return ErrorApplication::new(format!(
                    "Error loading configuration: {}",
                    error.to_string()
                ));
            }
        };

        // Spawn a thread that forces to update the ui every 200ms
        log::trace!("Spawning extra update thread");

        let ctx = ctx.egui_ctx.clone();
        std::thread::spawn(move || {
            let ctx_clone = ctx.clone();
            loop {
                std::thread::sleep(Duration::from_millis(200));
                ctx_clone.request_repaint();
            }
        });

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
        let application = Self {
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
            toasts: Toasts::new(),
        };

        Box::new(application)
    }

    fn get_server(&mut self, server: &Server) -> Option<&mut Server> {
        for owned_server in self.servers.iter_mut() {
            if owned_server == server {
                return Some(owned_server);
            }
        }

        None
    }

    // Get whether the current data is valid to allow the user click the Ok button
    fn login_form_is_valid(&mut self) -> bool {
        !(self.password.trim().is_empty()
            || self.root_password.trim().is_empty()
            || self.username.trim().is_empty())
    }

    fn show_notification(&mut self, message: WidgetText, kind: ToastKind) {
        self.toasts.add(Toast {
            kind,
            text: message,
            options: ToastOptions::default()
                .duration_in_seconds(3.0)
                .show_icon(true)
                .show_progress(true),
        });
    }
}
