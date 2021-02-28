// Frame Definitions
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct AVFrame {
    pub command: u8,
    pub ident: i64,
    pub frame: Vec<Vec<u8>>,
    // TODO: Should we be checksumming?
}
