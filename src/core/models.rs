/// Universal message representation for all chat sources.
/// All parsers convert their native format into this structure.
#[derive(Debug, Clone)]
pub struct InternalMessage {
    pub sender: String,
    pub content: String,
}

impl InternalMessage {
    pub fn new(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            sender: sender.into(),
            content: content.into(),
        }
    }
}
