use crate::{Runtime, VmError};
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
pub fn reset(rt: &mut Runtime, ft: FrameType) -> Result<()> {
    loop {
        let frame = rt
            .current_thread
            .runtime_stack
            .pop()
            .ok_or(VmError::RuntimeStackUnderflow)?;

        if frame.frame_type != ft {
            continue;
        }

        if let Some(address) = frame.address {
            rt.current_thread.pc = address;
        }

        rt.current_thread.env = frame.env;
        break;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use bytecode::{ByteCode, Environment, FrameType, StackFrame, Value};

    #[test]
    fn test_reset_restore_env() {
        let mut rt = Runtime::new(vec![ByteCode::RESET(FrameType::BlockFrame)]);

        let env_a = Environment::new_wrapped();
        let env_b = Environment::new_wrapped();
        let env_c = Environment::new_wrapped();
        env_c.borrow_mut().set("a", 42);

        let some_frame = StackFrame::new(FrameType::CallFrame, Rc::clone(&env_a));
        let block_frame = StackFrame::new(FrameType::BlockFrame, Rc::clone(&env_c));
        let call_frame = StackFrame::new(FrameType::CallFrame, Rc::clone(&env_b));

        rt.current_thread.runtime_stack.push(some_frame);
        rt.current_thread.runtime_stack.push(block_frame);
        rt.current_thread.runtime_stack.push(call_frame);

        assert!(rt.current_thread.runtime_stack.len() == 3);

        reset(&mut rt, FrameType::BlockFrame).unwrap();

        assert!(rt.current_thread.runtime_stack.len() == 1);
        assert_eq!(
            rt.current_thread.env.borrow().get(&"a".to_string()),
            Some(Value::Int(42))
        );
    }

    #[test]
    fn test_set_pc() {
        let mut rt = Runtime::new(vec![ByteCode::RESET(FrameType::BlockFrame)]);

        let env_a = Environment::new_wrapped();
        let env_b = Environment::new_wrapped();
        let env_c = Environment::new_wrapped();
        env_c.borrow_mut().set("a", 42);

        let some_frame = StackFrame::new(FrameType::CallFrame, Rc::clone(&env_a));
        let block_frame =
            StackFrame::new_with_address(FrameType::BlockFrame, Rc::clone(&env_c), 123);
        let call_frame = StackFrame::new(FrameType::CallFrame, Rc::clone(&env_b));

        rt.current_thread.runtime_stack.push(some_frame);
        rt.current_thread.runtime_stack.push(block_frame);
        rt.current_thread.runtime_stack.push(call_frame);

        assert!(rt.current_thread.runtime_stack.len() == 3);

        reset(&mut rt, FrameType::BlockFrame).unwrap();

        assert!(rt.current_thread.runtime_stack.len() == 1);
        assert_eq!(rt.current_thread.pc, 123);
    }
}
