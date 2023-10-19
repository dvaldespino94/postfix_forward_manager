use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq)]
pub enum AuthStatus {
    Unknown,
    Failed,
    Authenticated,
    InProgress,
}

impl Default for AuthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<bool> for AuthStatus {
    fn from(value: bool) -> Self {
        if value {
            Self::Authenticated
        } else {
            Self::Failed
        }
    }
}

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

    // Authentication status
    #[serde(skip)]
    pub auth_status: AuthStatus,

    // Flag to check if the server is still working
    #[serde(skip)]
    pub is_busy: bool,
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

    pub fn to_string_extended(&self) -> String {
        format!("{}:{}:{}", self.addr, self.port, self.config_path)
    }

    pub fn busy(&self)->bool{
        self.auth_status==AuthStatus::InProgress
    }
}

// Simple way to represent the server for debuging reasons mainly
impl Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}:{}", self.addr, self.port))
    }
}
