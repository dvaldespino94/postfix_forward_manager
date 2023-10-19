use std::{
    borrow::BorrowMut,
    collections::HashMap,
    error::Error,
    ffi::OsStr,
    io::{ErrorKind, Write},
    net::TcpStream,
    path::Path,
    time::Duration,
};

use ssh::LocalSession;
use tempfile::NamedTempFile;

use super::server::Server;

// Wrapper around the ssh connection with the servers
pub struct SSHWrapper {
    // The server's information
    server: Server,
    // Username to use in authentication
    username: String,
    // Password
    password: String,
    // root password to allow uploading the configuration
    root_password: String,
    // The actual ssh/scp client instance
    client: Option<LocalSession<TcpStream>>,
}

impl SSHWrapper {
    // Try to authenticate with the server
    pub fn authenticate(&mut self) -> Result<bool, Box<dyn Error>> {
        log::trace!(
            "Authenticating {}:{}@{}:{}",
            self.username,
            self.password,
            self.server.addr,
            self.server.port
        );

        // Try to create a session
        match ssh::create_session()
            // Using 2 seconds timeout seems to be enough
            .timeout(Some(Duration::from_secs(5)))
            // Set the username
            .username(&self.username)
            // password
            .password(&self.password)
            // Add this old (and apparently deprecated) pubkey algorithm
            .add_pubkey_algorithms(ssh::algorithm::PubKey::SshEd25519)
            // Connect to the server
            .connect(format!("{}:{}", self.server.addr, self.server.port))
        {
            // If the client connects successfully
            Ok(client) => {
                self.client = Some(client.run_local());
                return Ok(true);
            }
            Err(error) => match error {
                // If there was an authentication error just signal it
                ssh::SshError::AuthError => return Ok(false),
                // Other errors
                other => {
                    log::trace!("Unhandled error: {other:#?}");
                    return Err(Box::new(other));
                }
            },
        }
    }

    // Create a new wrapper
    pub fn new(server: Server, username: String, password: String, root_password: String) -> Self {
        Self {
            server,
            username,
            password,
            root_password,
            client: None,
        }
    }

    // Fetch the virtual users list from the server, parses it and returns it
    pub fn get_virtual_users(&mut self) -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
        // Create a temporary directory in the host so we can safely fetch the file from the server
        let temp_directory = tempfile::tempdir()?;

        // full path for the temporary directory plus the filename
        let local_path = temp_directory
            .path()
            .join(format!("temp_{}", self.server.addr.clone()));

        // String copy of the temporary file name
        let local_path_string = local_path.to_str().unwrap().to_owned();

        // Avoid querying if there is no client to query to
        if let Some(client) = self.client.borrow_mut() {
            // Create a scp client instance to download the file
            let scp = client.open_scp()?;

            // Download the file into the local temporary directory
            if let Err(error) =
                scp.download(&local_path.as_path(), &Path::new(&self.server.config_path))
            {
                // Show the error message in the logs
                log::error!(
                    "Can't download {}:{}, trying virtualuser: {error:?}",
                    self.server,
                    self.server.config_path
                );

                // Return the error to the caller
                return Err(Box::new(error));
            } else {
                // Show success message in the logs
                log::trace!("Got {}:{}", self.server, self.server.config_path);
            }
        }

        // Try to read the downloaded file
        log::trace!("Trying to open {local_path_string} to get the data");
        let data = std::fs::read_to_string(local_path.clone())?;

        let mut redirections: HashMap<String, Vec<String>> = Default::default();

        // Parse the file into the redirections hash
        for line in data.lines().map(|x| x.to_owned()) {
            // Skip comments
            if line.trim().starts_with("#") {
                continue;
            }

            // Split the line in two
            if let Some((email, rest)) = line.split_once(" ") {
                // Split the second part to obtain redirection targets
                let mut email_redirections: Vec<String> = rest
                    .split(|x: char| x.is_whitespace() || x == ',')
                    .map(|x| x.trim().to_owned())
                    .collect();
                // Remove empty entries
                email_redirections.retain(|redirection| !redirection.is_empty());
                // Remove duplicated entries
                email_redirections.dedup();

                // Insert the list into the redirections hash
                redirections.insert(email.to_owned(), email_redirections);
            } else {
                // Show an error
                log::error!(
                    "Server {} path {local_path_string} Skipping line '{line}'",
                    self.server
                );

                return Err(Box::new(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Error parsing file",
                )));
            }
        }

        Ok(redirections)
    }

    // Upload configurations to server
    pub fn upload_configuration(&mut self, server: Server) -> Result<(), Box<dyn Error>> {
        // Get the client
        let client = self.client.as_mut().ok_or(std::io::Error::new(
            ErrorKind::Other,
            "There is no SSH Client instance in this wrapper",
        ))?;

        let configuration_full_path = self.server.config_path.clone();
        // Get the configuration file's name
        let configuration_filename = Path::new(&configuration_full_path)
            .file_name()
            .unwrap_or(OsStr::new("virtual"))
            .to_str()
            .unwrap()
            .to_owned();

        // Generate the configuration's payload
        let payload = server.payload();

        // Create a local file to be sent to server
        let mut local_file = NamedTempFile::new()?;
        // Write the configuration payload and flush
        local_file.write_all(payload.as_bytes())?;
        local_file.flush()?;

        // Create a backup for the remote configuration
        let mut shell = client.open_shell()?;

        let _ = shell.write(
            format!(
                "cp '{}' {}\n",
                self.server.config_path,
                format!("~/{configuration_filename}_`date \"+%Y-%m-%d_%H-%M-%S\"`.bak")
            )
            .as_bytes(),
        )?;
        std::thread::sleep(Duration::from_secs(2));
        log::debug!("Result: {}", String::from_utf8(shell.read()?)?);

        // Create a scp client instance
        let scp = client.open_scp()?;
        // Upload the file
        scp.upload(
            &local_file.path().to_str().unwrap().to_owned(),
            &format!("/tmp/{configuration_filename}"),
        )?;

        // Create a shell so we can escalate privileges into root
        let mut shell = client.open_shell()?;
        // Read and discard the bash banner

        // Send and execute the command:
        shell.write(
            format!("su root -c 'cp /tmp/{configuration_filename} {configuration_full_path}; /etc/postfix/post_update'\n")
                .as_bytes(),
        )?;

        // Sleep for a while until the prompt is shown
        std::thread::sleep(Duration::from_millis(500));
        shell.write((self.root_password.clone() + "\n").as_bytes())?;
        // Sleep for a while until the file is properly modified
        std::thread::sleep(Duration::from_millis(500));

        // Create a scp client to download the allegedly uploaded configuration
        let scp = client.open_scp()?;
        // Create a temporary directory to get the configuration
        let local_tmp_dir = tempfile::tempdir()?;

        // Download the configuration
        scp.download(local_tmp_dir.path(), Path::new(&configuration_full_path))?;
        // Read the configuration into a string
        let data: String =
            std::fs::read_to_string(local_tmp_dir.path().join(configuration_filename))?;

        // Parse the lines
        let mut actual: Vec<String> = data.lines().map(|x| x.to_owned()).collect::<Vec<String>>();
        // Sort the lines
        actual.sort();

        // Do the same with the payload
        let mut payload = payload
            .lines()
            .map(|x| x.to_owned())
            .collect::<Vec<String>>();
        payload.sort();

        log::trace!(
            "Comparing:\n==========\n{}\n==========\n{}\n==========",
            actual.join("\n"),
            payload.join("\n")
        );

        // Comparer the actual data with the payload
        if actual != payload {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Configuration was not updated!",
            )));
        }

        Ok(())
    }
}
