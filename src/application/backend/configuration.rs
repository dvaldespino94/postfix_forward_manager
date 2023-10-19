use serde::Deserialize;

use super::server::Server;

// Store the app configuration (to be loaded from TOML, JSON, etc...)
#[derive(Deserialize)]
pub struct Configuration {
    // The default username as shown in the username field on login screen
    pub username: String,
    // The server list
    pub servers: Vec<Server>,
}

// Create default configuration (This should be deleted and application should sigsev or even BSOD on Windows
// when configuration fails to load)
impl Default for Configuration {
    fn default() -> Self {
        Self {
            username: "admin".to_owned(),
            servers: vec![Server {
                config_path: "/etc/postfix/virtual".to_owned(),
                addr: "127.0.0.1".to_owned(),
                port: 22,
                users: Default::default(),
                received_redirections: false,
                busy: false,
            }],
        }
    }
}
