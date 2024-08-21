use log::{error, info, warn};
use std::{future::Future, sync::Arc};
use tokio::sync::Mutex;

use crate::models::config::Config;

pub trait Functions {
    fn send_code(&self, client_id: i32, code: String) -> impl Future<Output = ()>;
}

pub trait Handlers {
    fn handle_update(
        &self,
        update: tdlib_rs::enums::Update,
        client_id: i32,
    ) -> impl Future<Output = ()>;
    fn handle_auth(
        &self,
        state: tdlib_rs::types::UpdateAuthorizationState,
        client_id: i32,
    ) -> impl Future<Output = ()>;
}

pub struct Events {
    pub config: Arc<Mutex<Config>>,
}

impl Functions for Events {
    async fn send_code(&self, client_id: i32, code: String) {
        let res = tdlib_rs::functions::check_authentication_code(code, client_id).await;
        match res {
            Err(tdlib_rs::types::Error { code, message }) => {
                error!("failed to send auth code: {}, {}", code, message);
            }
            _ => {}
        }
    }
}

impl Handlers for Events {
    async fn handle_auth(&self, state: tdlib_rs::types::UpdateAuthorizationState, client_id: i32) {
        match state.authorization_state {
            tdlib_rs::enums::AuthorizationState::Ready => {
                info!(target: "auth", "ready!");
            }
            tdlib_rs::enums::AuthorizationState::WaitTdlibParameters => {
                info!(target: "auth", "waiting for tdlib parameters");
                {
                    let cfg = self.config.lock().await;

                    let res = tdlib_rs::functions::set_tdlib_parameters(
                        false,
                        cfg.db_path.clone(),
                        cfg.files_path.clone(),
                        cfg.enc_key.clone(),
                        true,
                        true,
                        true,
                        false,
                        cfg.api_id,
                        cfg.api_hash.clone(),
                        cfg.user_agent.as_ref().unwrap().language.clone(),
                        cfg.user_agent.as_ref().unwrap().device.clone(),
                        String::new(),
                        String::from("1.0.15"),
                        client_id,
                    )
                    .await;

                    match res {
                        Err(tdlib_rs::types::Error { code, message }) => {
                            error!("set parameters error: {} with message: {}", code, message)
                        }
                        _ => {}
                    }
                }
            }
            tdlib_rs::enums::AuthorizationState::WaitPhoneNumber => {
                info!(target: "auth", "waiting for phone number");

                let cfg = self.config.lock().await;
                let _res = tdlib_rs::functions::set_authentication_phone_number(
                    String::from(cfg.phone.clone()),
                    None,
                    client_id,
                )
                .await;
            }
            tdlib_rs::enums::AuthorizationState::WaitCode(_) => {
                info!(target: "auth", "waiting for code");
            }
            _ => {
                warn!(target: "auth", "got unhandled auth event");
            }
        }
    }
    async fn handle_update(&self, update: tdlib_rs::enums::Update, client_id: i32) {
        match update {
            tdlib_rs::enums::Update::AuthorizationState(state) => {
                self.handle_auth(state, client_id).await;
            }
            tdlib_rs::enums::Update::NewMessage(_) => {} //info!("new message"),
            tdlib_rs::enums::Update::MessageSendAcknowledged(_) => info!("new message"),
            tdlib_rs::enums::Update::User(_) => {}
            tdlib_rs::enums::Update::Option(_) => {
                //info!("update option");
                //let _res = tdlib_rs::functions::set_option(opt.name, Some(opt.value), client_id).await;
                //info!("update option end");
            }
            tdlib_rs::enums::Update::ConnectionState(state) => match state.state {
                tdlib_rs::enums::ConnectionState::WaitingForNetwork => {
                    info!(target: "network", "waiting for network")
                }
                tdlib_rs::enums::ConnectionState::ConnectingToProxy => {
                    info!(target: "network","connecting to proxy")
                }
                tdlib_rs::enums::ConnectionState::Connecting => {
                    info!(target: "network","connecting")
                }
                tdlib_rs::enums::ConnectionState::Updating => {
                    info!(target: "network","updating")
                }
                tdlib_rs::enums::ConnectionState::Ready => {
                    info!(target: "network","ready")
                }
            },
            _ => {
                //info!(target: "main", "unhandled event");
            }
        }
    }
}
