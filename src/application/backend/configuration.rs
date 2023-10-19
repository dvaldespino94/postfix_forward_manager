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
