use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FrameType {
    BlockFrame,
    CallFrame,
}
