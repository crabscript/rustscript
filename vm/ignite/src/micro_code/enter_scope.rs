use anyhow::Result;
use bytecode::{FrameType, StackFrame, Symbol, Value, W};

use crate::{extend_environment, Runtime};

/// Create a new scope in the current environment. The new environment will be a child of the current
/// environment. All symbols in the new scope will be initialized to `Value::Unitialized`.
///
/// # Arguments
///
/// * `rt` - The runtime to create a new scope in.
///
/// * `syms` - The symbols to add to the new scope.
///
/// # Errors
///
/// Infallible.
pub fn enter_scope(mut rt: Runtime, syms: Vec<Symbol>) -> Result<Runtime> {
    let current_env = rt.current_thread.env.clone();

    // Preserve the current environment in a stack frame
    let frame = StackFrame::new(FrameType::BlockFrame, W(current_env));

    // Push the stack frame onto the runtime stack
    rt.current_thread.runtime_stack.push(frame);

    let uninitialized = syms
        .iter()
        .map(|_| Value::Unitialized)
        .collect::<Vec<Value>>();

    let current_env = rt.current_thread.env.clone();
    rt = extend_environment(rt, current_env, syms, uninitialized)?;

    Ok(rt)
}

#[cfg(test)]
mod tests {
    use bytecode::Value;

    use super::*;

    #[test]
    fn test_enter_scope() -> Result<()> {
        let mut rt = Runtime::new(vec![]);

        rt.current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow_mut()
            .set("a", 42);
        rt.current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow_mut()
            .set("b", 123);

        rt = enter_scope(rt, vec!["c".to_string(), "d".to_string()]).unwrap();

        assert_eq!(rt.current_thread.runtime_stack.len(), 1);
        assert!(rt
            .current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow()
            .parent
            .is_some());
        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"a".to_string())?,
            Value::Int(42)
        );
        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"b".to_string())?,
            Value::Int(123)
        );
        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"c".to_string())?,
            Value::Unitialized
        );
        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"d".to_string())?,
            Value::Unitialized
        );

        Ok(())
    }
}
