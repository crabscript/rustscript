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
pub fn reset(rt: &mut Runtime, t: FrameType) -> Result<()> {
    loop {
        let frame = rt
            .runtime_stack
            .pop()
            .ok_or(VmError::RuntimeStackUnderflow)?;

        if frame.frame_type != t {
            continue;
        }

        if let Some(address) = frame.address {
            rt.pc = address;
        }

        rt.env = frame.env;
        break;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Environment, Runtime, StackFrame};
    use bytecode::{ByteCode, FrameType, Value};
    use std::rc::Rc;

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

        rt.runtime_stack.push(some_frame);
        rt.runtime_stack.push(block_frame);
        rt.runtime_stack.push(call_frame);

        assert!(rt.runtime_stack.len() == 3);

        reset(&mut rt, FrameType::BlockFrame).unwrap();

        assert!(rt.runtime_stack.len() == 1);
        assert_eq!(rt.env.borrow().get(&"a".to_string()), Some(Value::Int(42)));
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
            StackFrame::new_with_address(FrameType::BlockFrame, 123, Rc::clone(&env_c));
        let call_frame = StackFrame::new(FrameType::CallFrame, Rc::clone(&env_b));

        rt.runtime_stack.push(some_frame);
        rt.runtime_stack.push(block_frame);
        rt.runtime_stack.push(call_frame);

        assert!(rt.runtime_stack.len() == 3);

        reset(&mut rt, FrameType::BlockFrame).unwrap();

        assert!(rt.runtime_stack.len() == 1);
        assert_eq!(rt.pc, 123);
    }
}
