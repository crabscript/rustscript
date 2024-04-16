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
pub fn exit_scope(mut rt: Runtime) -> Result<Runtime> {
    let prev_frame = rt
        .current_thread
        .runtime_stack
        .pop()
        .ok_or(VmError::RuntimeStackUnderflow)?;

    rt.current_thread.env = prev_frame.env.0;
    Ok(rt)
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use bytecode::{Environment, FrameType, StackFrame, Value, W};

    use super::*;

    #[test]
    fn test_exit_scope() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        let env_a = Environment::new_wrapped();
        env_a.borrow_mut().set("a", 42);
        let env_a_weak = Rc::downgrade(&env_a);

        let env_b = Environment::new_wrapped();
        env_b.borrow_mut().set("a", 123);
        let env_b_weak = Rc::downgrade(&env_b);

        rt.current_thread
            .runtime_stack
            .push(StackFrame::new(FrameType::BlockFrame, W(env_a_weak)));
        rt.current_thread.env = env_b_weak;

        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"a".to_string())?,
            Value::Int(123)
        );

        rt = exit_scope(rt).unwrap();

        assert_eq!(rt.current_thread.runtime_stack.len(), 0);
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
}
