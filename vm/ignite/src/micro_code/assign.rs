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
pub fn assign(rt: &mut Runtime, sym: Symbol) -> Result<()> {
    let val = rt
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;
    rt.env.borrow_mut().set(sym, val);
    Ok(())
}

#[cfg(test)]
mod tests {
    use bytecode::{Environment, Value};

    use super::*;

    #[test]
    fn test_assign() {
        let mut rt = Runtime::new(vec![]);
        rt.operand_stack.push(Value::Int(42));
        assign(&mut rt, "x".to_string()).unwrap();
        assert_eq!(rt.env.borrow().get(&"x".to_string()), Some(Value::Int(42)));
    }

    #[test]
    fn test_assign_with_parent() {
        let parent = Environment::new_wrapped();
        parent.borrow_mut().set("x", 42);
        let mut rt = Runtime::new(vec![]);
        let frame = Environment::new_wrapped();
        frame.borrow_mut().set_parent(parent);
        rt.env = frame;
        rt.operand_stack.push(Value::Int(43));
        assign(&mut rt, "y".to_string()).unwrap();
        assert_eq!(rt.env.borrow().get(&"x".to_string()), Some(Value::Int(42)));
        assert_eq!(rt.env.borrow().get(&"y".to_string()), Some(Value::Int(43)));
    }
}
