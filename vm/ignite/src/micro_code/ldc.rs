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
pub fn ldc(mut rt: Runtime, val: Value) -> Result<Runtime> {
    rt.current_thread.operand_stack.push(val);
    Ok(rt)
}

#[cfg(test)]
mod tests {
    use bytecode::Value;

    use super::*;

    #[test]
    fn test_ldc() {
        let mut rt = Runtime::new(vec![]);
        rt = ldc(rt, Value::Unit).unwrap();
        assert_eq!(rt.current_thread.operand_stack.pop().unwrap(), Value::Unit);

        rt = ldc(rt, Value::Int(42)).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Int(42)
        );

        rt = ldc(rt, Value::Float(42.0)).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Float(42.0)
        );

        rt = ldc(rt, Value::Bool(true)).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(true)
        );

        rt = ldc(rt, Value::String("hello world".into())).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::String("hello world".into())
        );
    }
}
