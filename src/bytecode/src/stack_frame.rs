use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::Environment;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FrameType {
    BlockFrame,
    CallFrame,
    ThreadFrame,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub frame_type: FrameType,
    pub address: Option<usize>,
    pub env: Rc<RefCell<Environment>>,
}

impl StackFrame {
    pub fn new(frame_type: FrameType, env: Rc<RefCell<Environment>>) -> Self {
        StackFrame {
            frame_type,
            address: None,
            env,
        }
    }

    pub fn new_with_address(
        frame_type: FrameType,
        address: usize,
        env: Rc<RefCell<Environment>>,
    ) -> Self {
        StackFrame {
            frame_type,
            address: Some(address),
            env,
        }
    }
}
