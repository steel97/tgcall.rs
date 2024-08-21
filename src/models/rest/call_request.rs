use serde::Deserialize;

#[derive(Deserialize)]
pub struct CallRequest {
    pub key: String,
    pub username: String,
}
