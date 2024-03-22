use crate::{Runtime, VmError};
use anyhow::Result;
use bytecode::Symbol;

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
        .env
        .borrow()
        .get(&sym)
        .ok_or_else(|| VmError::SymbolNotFound(sym.clone()))?;
    rt.operand_stack.push(val);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Environment;
    use bytecode::Value;

    #[test]
    fn test_ld() {
        let mut rt = Runtime::new(vec![]);
        rt.env.borrow_mut().set("x".to_string(), 42);
        ld(&mut rt, "x".to_string()).unwrap();
        assert_eq!(rt.operand_stack.pop(), Some(Value::Int(42)));
    }

    #[test]
    fn test_ld_with_parent() {
        let parent = Environment::new_wrapped();
        parent.borrow_mut().set("x", 42);
        let mut rt = Runtime::new(vec![]);
        let frame = Environment::new_wrapped();
        frame.borrow_mut().set_parent(parent);
        rt.env = frame;
        ld(&mut rt, "x".to_string()).unwrap();
        assert_eq!(rt.operand_stack.pop(), Some(Value::Int(42)));
    }
}
