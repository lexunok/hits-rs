use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CustomMessage {
    pub message: String,
}
