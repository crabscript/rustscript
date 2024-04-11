use anyhow::Result;

use crate::{Runtime, VmError};

/// Exit the current scope and restores the previous environment.
///
/// # Arguments
///
/// * `rt` - The runtime to exit the current scope in.
///
/// # Errors
///
/// If the runtime stack is empty.
pub fn exit_scope(rt: &mut Runtime) -> Result<()> {
    let prev_frame = rt
        .current_thread
        .runtime_stack
        .pop()
        .ok_or(VmError::RuntimeStackUnderflow)?;

    rt.current_thread.env = prev_frame.env;
    Ok(())
}

#[cfg(test)]
mod tests {
    use bytecode::{Environment, FrameType, StackFrame, Value};

    use super::*;

    #[test]
    fn test_exit_scope() {
        let mut rt = Runtime::new(vec![]);
        let env_a = Environment::new_wrapped();
        env_a.borrow_mut().set("a", 42);

        let env_b = Environment::new_wrapped();
        env_b.borrow_mut().set("a", 123);

        rt.current_thread
            .runtime_stack
            .push(StackFrame::new(FrameType::BlockFrame, env_a));
        rt.current_thread.env = env_b;

        assert_eq!(
            rt.current_thread.env.borrow().get(&"a".to_string()),
            Some(Value::Int(123))
        );

        exit_scope(&mut rt).unwrap();

        assert_eq!(rt.current_thread.runtime_stack.len(), 0);
        assert_eq!(
            rt.current_thread.env.borrow().get(&"a".to_string()),
            Some(Value::Int(42))
        );
    }
}
