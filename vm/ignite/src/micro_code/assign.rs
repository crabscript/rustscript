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
    rt.frame.borrow_mut().set(sym, val);
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
        assert_eq!(
            rt.frame.borrow().get(&"x".to_string()),
            Some(Value::Int(42))
        );
    }

    #[test]
    fn test_assign_with_parent() {
        let parent = Frame::new_wrapped();
        parent.borrow_mut().set("x", 42);
        let mut rt = Runtime::new(vec![]);
        let frame = Frame::new_wrapped();
        frame.borrow_mut().set_parent(parent);
        rt.frame = frame;
        rt.operand_stack.push(Value::Int(43));
        assign(&mut rt, "y".to_string()).unwrap();
        assert_eq!(
            rt.frame.borrow().get(&"x".to_string()),
            Some(Value::Int(42))
        );
        assert_eq!(
            rt.frame.borrow().get(&"y".to_string()),
            Some(Value::Int(43))
        );
    }
}
