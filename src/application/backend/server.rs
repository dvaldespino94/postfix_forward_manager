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

#[derive(Clone, Debug, PartialEq)]
pub enum UsersStatus {
    Unknown,
    Downloading,
    Idle,
    Uploading,
}

impl Default for UsersStatus {
    fn default() -> Self {
        Self::Unknown
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

    // Authentication status
    #[serde(skip)]
    pub auth_status: AuthStatus,

    // Authentication status
    #[serde(skip)]
    pub users_status: UsersStatus,
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

    pub fn busy(&self) -> bool {
        self.auth_status == AuthStatus::InProgress
            || self.users_status == UsersStatus::Downloading
            || self.users_status == UsersStatus::Uploading
    }
}

// Simple way to represent the server for debuging reasons mainly
impl Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}:{}", self.addr, self.port))
    }
}
