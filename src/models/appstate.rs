use std::sync::Arc;
use tokio::sync::Mutex;

use crate::telegram::events::Events;

use super::config::Config;

pub struct AppState {
    pub config: Arc<Mutex<Config>>,
    pub client_id: i32,
    pub events: Arc<Mutex<Events>>,
}
