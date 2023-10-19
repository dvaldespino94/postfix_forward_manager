use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use serde::Deserialize;

// Struct to hold information about the servers and throw it around between threads
#[derive(Clone, Deserialize)]
pub struct Server {
    // Path for the configuration on the server (tipically /etc/postfix/virtual or .../virtualuser)
    pub config_path: String,
    // Addres of the server (ip or dns, it resolves autimatically)
    pub addr: String,
    // Port for the ssh connection, tipically 22
    pub port: u16,

    // Store the loaded data from the server, it's not serialized so it must be skipped
    #[serde(skip)]
    pub users: HashMap<String, Vec<String>>,
    // Flags if the data has already been filled from the server
    #[serde(skip)]
    pub received_redirections: bool,
    // Flag to check if the server is still working
    #[serde(skip)]
    pub busy: bool,
}

// Compare two server instances, only taking into account the path, address and port
impl PartialEq for Server {
    fn eq(&self, other: &Self) -> bool {
        self.config_path == other.config_path && self.addr == other.addr && self.port == other.port
    }
}

impl Server {
    // Generate the payload to be uploaded to the server, based on the user-defined info
    pub fn payload(&self) -> String {
        let mut payload = String::new();
        for (k, entries) in self.users.iter() {
            payload += &format!("{k} {}\n", entries.join(" "));
        }

        payload
    }
}

// Simple way to represent the server for debuging reasons mainly
impl Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}:{}", self.addr, self.port))
    }
}
