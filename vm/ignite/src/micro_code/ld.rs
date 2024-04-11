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
pub fn ld(rt: &mut Runtime, sym: Symbol) -> Result<()> {
    let val = rt
        .current_thread
        .env
        .borrow()
        .get(&sym)
        .ok_or_else(|| VmError::UnboundedName(sym.clone()))?;
    rt.current_thread.operand_stack.push(val);
    Ok(())
}

#[cfg(test)]
mod tests {
    use bytecode::{Environment, Value};

    use super::*;

    #[test]
    fn test_ld() {
        let mut rt = Runtime::new(vec![]);
        rt.current_thread.env.borrow_mut().set("x".to_string(), 42);
        ld(&mut rt, "x".to_string()).unwrap();
        assert_eq!(rt.current_thread.operand_stack.pop(), Some(Value::Int(42)));
    }

    #[test]
    fn test_ld_with_parent() {
        let parent = Environment::new_wrapped();
        parent.borrow_mut().set("x", 42);
        let mut rt = Runtime::new(vec![]);
        let frame = Environment::new_wrapped();
        frame.borrow_mut().set_parent(parent);
        rt.current_thread.env = frame;
        ld(&mut rt, "x".to_string()).unwrap();
        assert_eq!(rt.current_thread.operand_stack.pop(), Some(Value::Int(42)));
    }
}
