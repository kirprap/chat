use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub username: String,
    pub content: String,
    pub message_type: String, // "message", "join", "leave"
}