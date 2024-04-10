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
pub fn assign(rt: &mut Runtime, sym: Symbol) -> Result<()> {
    let val = rt
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;
    rt.env.borrow_mut().update(sym, val)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use bytecode::{Environment, Value};

    use super::*;

    #[test]
    fn test_assign() {
        let mut rt = Runtime::new(vec![]);
        rt.env.borrow_mut().set("x", Value::Unitialized);
        rt.operand_stack.push(Value::Int(42));

        assign(&mut rt, "x".to_string()).unwrap();

        assert_ne!(
            rt.env.borrow().get(&"x".to_string()),
            Some(Value::Unitialized)
        );
        assert_eq!(rt.env.borrow().get(&"x".to_string()), Some(Value::Int(42)));
    }

    #[test]
    fn test_assign_with_parent() {
        let mut rt = Runtime::new(vec![]);

        let parent_env = Environment::new_wrapped();
        parent_env.borrow_mut().set("x", 42);

        let child_env = Environment::new_wrapped();
        child_env.borrow_mut().set_parent(Rc::clone(&parent_env));
        child_env.borrow_mut().set("y", Value::Unitialized);

        rt.env = Rc::clone(&child_env);
        rt.operand_stack.push(Value::Int(123));
        assign(&mut rt, "x".to_string()).unwrap();

        assert_eq!(
            parent_env.borrow().get(&"x".to_string()),
            Some(Value::Int(123))
        );
        // The child environment should not be updated.
        assert!(!child_env.borrow().env.contains_key(&"x".to_string()));

        rt.operand_stack.push(Value::Int(789));
        assign(&mut rt, "y".to_string()).unwrap();

        assert!(parent_env.borrow().get(&"y".to_string()).is_none());
        assert_eq!(
            child_env.borrow().get(&"y".to_string()),
            Some(Value::Int(789))
        );
        assert_eq!(rt.env.borrow().get(&"y".to_string()), Some(Value::Int(789)));
    }
}
