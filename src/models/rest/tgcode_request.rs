use serde::Deserialize;

#[derive(Deserialize)]
pub struct TgCodeRequest {
    pub key: String,
    pub code: String,
}
