use crate::Environment;
use bytecode::FrameType;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
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
