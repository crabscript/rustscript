use anyhow::Result;
use bytecode::Value;

use crate::Thread;

/// Loads a constant value onto the stack.
///
/// # Arguments
///
/// * `rt` - The runtime to load the constant onto.
///
/// * `val` - The value to load onto the stack.
///
/// # Errors
///
/// Infallible.
pub fn ldc(t: &mut Thread, val: Value) -> Result<()> {
    t.operand_stack.push(val);
    Ok(())
}

#[cfg(test)]
mod tests {
    use bytecode::Value;

    use super::*;
    use crate::Thread;

    #[test]
    fn test_ldc() {
        let mut t = Thread::new(vec![]);
        ldc(&mut t, Value::Unit).unwrap();
        assert_eq!(t.operand_stack.pop().unwrap(), Value::Unit);

        ldc(&mut t, Value::Int(42)).unwrap();
        assert_eq!(t.operand_stack.pop().unwrap(), Value::Int(42));

        ldc(&mut t, Value::Float(42.0)).unwrap();
        assert_eq!(t.operand_stack.pop().unwrap(), Value::Float(42.0));

        ldc(&mut t, Value::Bool(true)).unwrap();
        assert_eq!(t.operand_stack.pop().unwrap(), Value::Bool(true));

        ldc(&mut t, Value::String("hello world".into())).unwrap();
        assert_eq!(
            t.operand_stack.pop().unwrap(),
            Value::String("hello world".into())
        );
    }
}
