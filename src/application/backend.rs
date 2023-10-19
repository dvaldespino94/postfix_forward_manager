use self::{
    messages::{QueryMessage, ResponseMessage},
    sshwrapper::SSHWrapper,
};
use std::{
    collections::HashMap,
    ops::ControlFlow,
    sync::mpsc::{Receiver, Sender},
};

pub mod configuration;
pub mod messages;
pub mod server;
pub mod sshwrapper;

pub fn backend_loop(
    rx: &Receiver<QueryMessage>,
    tx: &Sender<ResponseMessage>,
    ssh_sessions: &mut HashMap<String, SSHWrapper>,
) -> ControlFlow<()> {
    match rx.recv() {
        Ok(msg) => match msg {
            QueryMessage::QueryVirtualUsers(server) => {
                log::trace!("Requested virtual users for server {server}");

                if let Some(session) = ssh_sessions.get_mut(&server.to_string_extended()) {
                    match session.get_virtual_users() {
                        Ok(users) => {
                            let _ = tx.send(ResponseMessage::GotVirtualUsers { server, users });
                        }
                        Err(error) => {
                            log::error!("Error getting virtual users: {error:?}");
                            let _ = tx.send(ResponseMessage::QueryVirtualUsersResult {
                                server,
                                error: error.to_string(),
                            });
                        }
                    }
                }
            }
            QueryMessage::Authenticate {
                username,
                password,
                servers,
                root_password,
            } => {
                for server in servers.iter() {
                    let mut wrapper = SSHWrapper::new(
                        server.clone(),
                        username.clone(),
                        password.clone(),
                        root_password.clone(),
                    );

                    match wrapper.authenticate() {
                        Ok(result) => {
                            if result {
                                let _ = tx.send(ResponseMessage::AuthenticationResult {
                                    server: server.clone(),
                                    success: true,
                                    error: None,
                                });
                            } else {
                                let _ = tx.send(ResponseMessage::AuthenticationResult {
                                    server: server.to_owned(),
                                    success: false,
                                    error: None,
                                });
                            }
                        }
                        Err(error) => {
                            log::error!("Authentication error: {error:?}");
                            let _ = tx.send(ResponseMessage::AuthenticationResult {
                                server: server.to_owned(),
                                success: false,
                                error: Some(error.to_string()),
                            });
                        }
                    }
                    ssh_sessions.insert(server.to_string_extended(), wrapper);
                }
            }
            QueryMessage::UpdateVirtualUsers(server) => {
                let server = &server;
                for (key, session) in ssh_sessions {
                    if &server.to_string_extended() == key {
                        let error = session.upload_configuration(server.clone()).err();

                        let _ = tx.send(ResponseMessage::ServerUploadResult {
                            server: server.clone(),
                            error: error.map(|x| x.to_string()),
                        });
                        break;
                    }
                }
            }
        },
        Err(_) => return ControlFlow::Break(()),
    }

    ControlFlow::Continue(())
}
