use anyhow::Result;
use bytecode::Value;

use crate::Runtime;

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
pub fn ldc(rt: &mut Runtime, val: Value) -> Result<()> {
    rt.operand_stack.push(val);
    Ok(())
}

#[cfg(test)]
mod tests {
    use bytecode::Value;

    use super::*;
    use crate::Runtime;

    #[test]
    fn test_ldc() {
        let mut rt = Runtime::new(vec![]);
        ldc(&mut rt, Value::Unit).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Unit);

        ldc(&mut rt, Value::Int(42)).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Int(42));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Float(42.0));

        ldc(&mut rt, Value::Bool(true)).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Bool(true));

        ldc(&mut rt, Value::String("hello world".into())).unwrap();
        assert_eq!(
            rt.operand_stack.pop().unwrap(),
            Value::String("hello world".into())
        );
    }
}
