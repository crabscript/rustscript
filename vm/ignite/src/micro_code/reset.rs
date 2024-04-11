use crate::{Thread, VmError};
use anyhow::Result;
use bytecode::FrameType;

/// Reset the runtime to the last frame of the given type. This will pop all frames up to and including
/// the last frame of the given type.
///
/// # Arguments
///
/// * `rt` - The runtime to reset.
///
/// * `t` - The type of frame to reset to.
///
/// # Errors
///
/// If the runtime stack underflows. i.e. there are no frames of the given type.
pub fn reset(t: &mut Thread, ft: FrameType) -> Result<()> {
    loop {
        let frame = t
            .runtime_stack
            .pop()
            .ok_or(VmError::RuntimeStackUnderflow)?;

        if frame.frame_type != ft {
            continue;
        }

        if let Some(address) = frame.address {
            t.pc = address;
        }

        t.env = frame.env;
        break;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use bytecode::{ByteCode, Environment, FrameType, StackFrame, Value};

    use crate::Thread;

    #[test]
    fn test_reset_restore_env() {
        let mut t = Thread::new(vec![ByteCode::RESET(FrameType::BlockFrame)]);

        let env_a = Environment::new_wrapped();
        let env_b = Environment::new_wrapped();
        let env_c = Environment::new_wrapped();
        env_c.borrow_mut().set("a", 42);

        let some_frame = StackFrame::new(FrameType::CallFrame, Rc::clone(&env_a));
        let block_frame = StackFrame::new(FrameType::BlockFrame, Rc::clone(&env_c));
        let call_frame = StackFrame::new(FrameType::CallFrame, Rc::clone(&env_b));

        t.runtime_stack.push(some_frame);
        t.runtime_stack.push(block_frame);
        t.runtime_stack.push(call_frame);

        assert!(t.runtime_stack.len() == 3);

        reset(&mut t, FrameType::BlockFrame).unwrap();

        assert!(t.runtime_stack.len() == 1);
        assert_eq!(t.env.borrow().get(&"a".to_string()), Some(Value::Int(42)));
    }

    #[test]
    fn test_set_pc() {
        let mut t = Thread::new(vec![ByteCode::RESET(FrameType::BlockFrame)]);

        let env_a = Environment::new_wrapped();
        let env_b = Environment::new_wrapped();
        let env_c = Environment::new_wrapped();
        env_c.borrow_mut().set("a", 42);

        let some_frame = StackFrame::new(FrameType::CallFrame, Rc::clone(&env_a));
        let block_frame =
            StackFrame::new_with_address(FrameType::BlockFrame, 123, Rc::clone(&env_c));
        let call_frame = StackFrame::new(FrameType::CallFrame, Rc::clone(&env_b));

        t.runtime_stack.push(some_frame);
        t.runtime_stack.push(block_frame);
        t.runtime_stack.push(call_frame);

        assert!(t.runtime_stack.len() == 3);

        reset(&mut t, FrameType::BlockFrame).unwrap();

        assert!(t.runtime_stack.len() == 1);
        assert_eq!(t.pc, 123);
    }
}
