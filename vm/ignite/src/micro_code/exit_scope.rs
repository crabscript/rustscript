use anyhow::Result;

use crate::{Thread, VmError};

/// Exit the current scope and restores the previous environment.
///
/// # Arguments
///
/// * `rt` - The runtime to exit the current scope in.
///
/// # Errors
///
/// If the runtime stack is empty.
pub fn exit_scope(t: &mut Thread) -> Result<()> {
    let prev_frame = t
        .runtime_stack
        .pop()
        .ok_or(VmError::RuntimeStackUnderflow)?;

    t.env = prev_frame.env;
    Ok(())
}

#[cfg(test)]
mod tests {
    use bytecode::{Environment, FrameType, StackFrame, Value};

    use super::*;

    #[test]
    fn test_exit_scope() {
        let mut t = Thread::new(vec![]);
        let env_a = Environment::new_wrapped();
        env_a.borrow_mut().set("a", 42);

        let env_b = Environment::new_wrapped();
        env_b.borrow_mut().set("a", 123);

        t.runtime_stack
            .push(StackFrame::new(FrameType::BlockFrame, env_a));
        t.env = env_b;

        assert_eq!(t.env.borrow().get(&"a".to_string()), Some(Value::Int(123)));

        exit_scope(&mut t).unwrap();

        assert_eq!(t.runtime_stack.len(), 0);
        assert_eq!(t.env.borrow().get(&"a".to_string()), Some(Value::Int(42)));
    }
}
