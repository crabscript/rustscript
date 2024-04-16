use anyhow::Result;
use bytecode::Symbol;

use crate::{Runtime, VmError};

/// Load a value from a symbol.
///
/// # Arguments
///
/// * `rt` - The runtime to execute the instruction on.
///
/// * `sym` - The symbol to load the value from.
///
/// # Errors
///
/// If the symbol is not found.
pub fn ld(mut rt: Runtime, sym: Symbol) -> Result<Runtime> {
    let val = rt
        .current_thread
        .env
        .upgrade()
        .ok_or(VmError::EnvironmentDroppedError)?
        .borrow()
        .get(&sym)?;

    rt.current_thread.operand_stack.push(val);
    Ok(rt)
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use bytecode::{Environment, Value};

    use super::*;

    #[test]
    fn test_ld() {
        let mut rt = Runtime::new(vec![]);
        rt.current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow_mut()
            .set("x".to_string(), 42);
        rt = ld(rt, "x".to_string()).unwrap();
        assert_eq!(rt.current_thread.operand_stack.pop(), Some(Value::Int(42)));
    }

    #[test]
    fn test_ld_with_parent() {
        let parent = Environment::new_wrapped();
        let parent_weak = Rc::downgrade(&parent);
        parent.borrow_mut().set("x", 42);
        let mut rt = Runtime::new(vec![]);
        let env = Environment::new_wrapped();
        let env_weak = Rc::downgrade(&env);
        env.borrow_mut().set_parent(parent_weak);
        rt.current_thread.env = env_weak;
        rt = ld(rt, "x".to_string()).unwrap();
        assert_eq!(rt.current_thread.operand_stack.pop(), Some(Value::Int(42)));
    }
}
