use std::collections::HashMap;


use super::server::Server;

// Messages sent from the frontend to the backend
pub enum QueryMessage {
    // Querythe virtual users
    QueryVirtualUsers(Server),
    // Update the virtual users
    UpdateVirtualUsers(Server),
    // Try to authenticate
    Authenticate {
        username: String,
        password: String,
        servers: Vec<Server>,
        root_password: String,
    },
}

// Response messages sent from the backend to the frontend
pub enum ResponseMessage {
    // Got some virtual users
    GotVirtualUsers {
        server: Server,
        users: HashMap<String, Vec<String>>,
    },
    // The virtual users query returned some errors
    QueryVirtualUsersResult {
        server: Server,
        error: String,
    },
    // The server upload process finished
    ServerUploadResult {
        server: Server,
        error: Option<String>,
    },
    // Result for the authentication process
    AuthenticationResult {
        server: Server,
        success: bool,
        error: Option<String>,
    },
}
