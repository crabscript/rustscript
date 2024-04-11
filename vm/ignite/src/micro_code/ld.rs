use anyhow::Result;
use bytecode::Symbol;

use crate::{Thread, VmError};

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
pub fn ld(t: &mut Thread, sym: Symbol) -> Result<()> {
    let val = t
        .env
        .borrow()
        .get(&sym)
        .ok_or_else(|| VmError::UnboundedName(sym.clone()))?;
    t.operand_stack.push(val);
    Ok(())
}

#[cfg(test)]
mod tests {
    use bytecode::{Environment, Value};

    use super::*;

    #[test]
    fn test_ld() {
        let mut t = Thread::new(vec![]);
        t.env.borrow_mut().set("x".to_string(), 42);
        ld(&mut t, "x".to_string()).unwrap();
        assert_eq!(t.operand_stack.pop(), Some(Value::Int(42)));
    }

    #[test]
    fn test_ld_with_parent() {
        let parent = Environment::new_wrapped();
        parent.borrow_mut().set("x", 42);
        let mut rt = Thread::new(vec![]);
        let frame = Environment::new_wrapped();
        frame.borrow_mut().set_parent(parent);
        rt.env = frame;
        ld(&mut rt, "x".to_string()).unwrap();
        assert_eq!(rt.operand_stack.pop(), Some(Value::Int(42)));
    }
}
