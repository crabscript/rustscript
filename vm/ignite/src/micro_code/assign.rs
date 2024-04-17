use anyhow::{Ok, Result};
use bytecode::Symbol;

use crate::{Runtime, VmError};

/// Assign a value to a symbol.
///
/// # Arguments
///
/// * `rt` - The runtime to execute the instruction on.
///
/// * `sym` - The symbol to assign the value to.
///
/// # Errors
///
/// If the stack is empty.
/// If the symbol is not found in the environment chain.
pub fn assign(mut rt: Runtime, sym: Symbol) -> Result<Runtime> {
    let val = rt
        .current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;
    rt.current_thread
        .env
        .upgrade()
        .ok_or(VmError::EnvironmentDroppedError)?
        .borrow_mut()
        .update(sym, val)?;

    Ok(rt)
}

#[cfg(test)]
mod tests {
    use bytecode::{weak_clone, Environment, Value};

    use super::*;

    #[test]
    fn test_assign() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        rt.current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow_mut()
            .set("x", Value::Unitialized);
        rt.current_thread.operand_stack.push(Value::Int(42));

        rt = assign(rt, "x".to_string()).unwrap();

        assert_ne!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"x".to_string())?,
            Value::Unitialized
        );
        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"x".to_string())?,
            Value::Int(42)
        );

        Ok(())
    }

    #[test]
    fn test_assign_with_parent() -> Result<()> {
        let mut rt = Runtime::new(vec![]);

        let parent_env = Environment::new_wrapped();
        let parent_weak = weak_clone(&parent_env);
        parent_env.borrow_mut().set("x", 42);

        let child_env = Environment::new_wrapped();
        child_env.borrow_mut().set_parent(parent_weak);
        child_env.borrow_mut().set("y", Value::Unitialized);
        let child_weak = weak_clone(&child_env);

        rt.current_thread.env = child_weak;
        rt.current_thread.operand_stack.push(Value::Int(123));
        rt = assign(rt, "x".to_string()).unwrap();

        assert_eq!(parent_env.borrow().get(&"x".to_string())?, Value::Int(123));
        // The child environment should not be updated.
        assert!(!child_env.borrow().env.contains_key(&"x".to_string()));

        rt.current_thread.operand_stack.push(Value::Int(789));
        rt = assign(rt, "y".to_string()).unwrap();

        assert!(parent_env.borrow().get(&"y".to_string()).is_err());
        assert_eq!(child_env.borrow().get(&"y".to_string())?, Value::Int(789));
        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"y".to_string())?,
            Value::Int(789)
        );

        Ok(())
    }
}
