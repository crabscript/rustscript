use serde::{Deserialize, Serialize};

use crate::EnvWeak;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FrameType {
    BlockFrame,
    CallFrame,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub frame_type: FrameType,
    pub address: Option<usize>,
    pub env: EnvWeak,
}

impl StackFrame {
    pub fn new(frame_type: FrameType, env: EnvWeak) -> Self {
        StackFrame {
            frame_type,
            address: None,
            env,
        }
    }

    pub fn new_with_address(frame_type: FrameType, env: EnvWeak, address: usize) -> Self {
        StackFrame {
            frame_type,
            address: Some(address),
            env,
        }
    }
}
