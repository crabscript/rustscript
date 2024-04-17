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
pub fn reset(mut rt: Runtime, ft: FrameType) -> Result<Runtime> {
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

        rt.current_thread.env = frame.env.0;
        break;
    }

    Ok(rt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode::{weak_clone, ByteCode, Environment, FrameType, StackFrame, Value, W};

    #[test]
    fn test_reset_restore_env() -> Result<()> {
        let mut rt = Runtime::new(vec![ByteCode::RESET(FrameType::BlockFrame)]);

        let env_a = Environment::new_wrapped();
        let env_b = Environment::new_wrapped();
        let env_c = Environment::new_wrapped();
        let env_a_weak = W(weak_clone(&env_a));
        let env_b_weak = W(weak_clone(&env_c));
        let env_c_weak = W(weak_clone(&env_b));
        env_c.borrow_mut().set("a", 42);

        let some_frame = StackFrame::new(FrameType::CallFrame, env_a_weak);
        let block_frame = StackFrame::new(FrameType::BlockFrame, env_b_weak);
        let call_frame = StackFrame::new(FrameType::CallFrame, env_c_weak);

        rt.current_thread.runtime_stack.push(some_frame);
        rt.current_thread.runtime_stack.push(block_frame);
        rt.current_thread.runtime_stack.push(call_frame);

        assert!(rt.current_thread.runtime_stack.len() == 3);

        rt = reset(rt, FrameType::BlockFrame).unwrap();

        assert!(rt.current_thread.runtime_stack.len() == 1);
        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"a".to_string())?,
            Value::Int(42)
        );

        Ok(())
    }

    #[test]
    fn test_set_pc() {
        let mut rt = Runtime::new(vec![ByteCode::RESET(FrameType::BlockFrame)]);

        let env_a = Environment::new_wrapped();
        let env_b = Environment::new_wrapped();
        let env_c = Environment::new_wrapped();
        let env_a_weak = W(weak_clone(&env_a));
        let env_b_weak = W(weak_clone(&env_c));
        let env_c_weak = W(weak_clone(&env_b));
        env_c.borrow_mut().set("a", 42);

        let some_frame = StackFrame::new(FrameType::CallFrame, env_a_weak);
        let block_frame = StackFrame::new_with_address(FrameType::BlockFrame, env_c_weak, 123);
        let call_frame = StackFrame::new(FrameType::CallFrame, env_b_weak);

        rt.current_thread.runtime_stack.push(some_frame);
        rt.current_thread.runtime_stack.push(block_frame);
        rt.current_thread.runtime_stack.push(call_frame);

        assert!(rt.current_thread.runtime_stack.len() == 3);

        rt = reset(rt, FrameType::BlockFrame).unwrap();

        assert!(rt.current_thread.runtime_stack.len() == 1);
        assert_eq!(rt.current_thread.pc, 123);
    }
}
