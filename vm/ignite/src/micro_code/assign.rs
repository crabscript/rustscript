use crate::{Runtime, VmError};
use anyhow::{Ok, Result};
use bytecode::Symbol;

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
    let val = rt.operand_stack.pop().ok_or(VmError::StackUnderflow)?;
    rt.frame.set(sym, val);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Frame;
    use bytecode::Value;

    #[test]
    fn test_assign() {
        let mut rt = Runtime::new(vec![]);
        rt.operand_stack.push(Value::Int(42));
        assign(&mut rt, "x".to_string()).unwrap();
        assert_eq!(rt.frame.get(&"x".to_string()), Some(&Value::Int(42)));
    }

    #[test]
    fn test_assign_with_parent() {
        let mut parent = Frame::new(None);
        parent.set("x", 42);
        let mut rt = Runtime::new(vec![]);
        rt.frame = Frame::new(Some(Box::new(parent)));
        rt.operand_stack.push(Value::Int(43));
        assign(&mut rt, "y".to_string()).unwrap();
        assert_eq!(rt.frame.get(&"x".to_string()), Some(&Value::Int(42)));
        assert_eq!(rt.frame.get(&"y".to_string()), Some(&Value::Int(43)));
    }
}
