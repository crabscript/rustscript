use std::rc::Rc;

use anyhow::Result;
use bytecode::{FrameType, StackFrame, Symbol, Value};

use crate::{thread::extend_environment, Thread};

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
pub fn enter_scope(t: &mut Thread, syms: Vec<Symbol>) -> Result<()> {
    let current_env = Rc::clone(&t.env);

    // Preserve the current environment in a stack frame
    let frame = StackFrame::new(FrameType::BlockFrame, Rc::clone(&current_env));

    // Push the stack frame onto the runtime stack
    t.runtime_stack.push(frame);

    let uninitialized = syms
        .iter()
        .map(|_| Value::Unitialized)
        .collect::<Vec<Value>>();

    extend_environment(t, syms, uninitialized)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use bytecode::Value;

    use super::*;

    #[test]
    fn test_enter_scope() {
        let mut t = Thread::new(vec![]);
        let env = Rc::clone(&t.env);

        t.env.borrow_mut().set("a", 42);
        t.env.borrow_mut().set("b", 123);

        enter_scope(&mut t, vec!["c".to_string(), "d".to_string()]).unwrap();

        assert_eq!(t.runtime_stack.len(), 1);
        assert_eq!(t.env.borrow().get(&"a".to_string()), Some(Value::Int(42)));
        assert_eq!(t.env.borrow().get(&"b".to_string()), Some(Value::Int(123)));
        assert_eq!(
            t.env.borrow().get(&"c".to_string()),
            Some(Value::Unitialized)
        );
        assert_eq!(
            t.env.borrow().get(&"d".to_string()),
            Some(Value::Unitialized)
        );

        t.env = env;

        assert_eq!(t.env.borrow().get(&"a".to_string()), Some(Value::Int(42)));
        assert_eq!(t.env.borrow().get(&"b".to_string()), Some(Value::Int(123)));
        assert_eq!(t.env.borrow().get(&"c".to_string()), None);
        assert_eq!(t.env.borrow().get(&"d".to_string()), None);
    }
}
