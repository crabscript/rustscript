use crate::{Runtime, VmError};
use anyhow::Result;

/// Pops a value off the stack.
///
/// # Arguments
///
/// * `rt` - The runtime to pop the value off of.
///
/// # Errors
///
/// If the stack is empty.
pub fn pop(rt: &mut Runtime) -> Result<()> {
    rt.operand_stack.pop().ok_or(VmError::StackUnderflow)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::micro_code::ldc;
    use crate::Runtime;
    use bytecode::Value;

    #[test]
    fn test_pop() {
        let mut rt = Runtime::new(vec![]);
        ldc(&mut rt, Value::Unit).unwrap();
        pop(&mut rt).unwrap();
        assert_eq!(rt.operand_stack.len(), 0);

        let vals = vec![
            Value::Unit,
            Value::Int(42),
            Value::Float(42.0),
            Value::Bool(true),
            Value::String("hello world".into()),
        ];
        let val_len = vals.len();
        let mut rt = Runtime::new(vec![]);
        for val in vals {
            ldc(&mut rt, val).unwrap();
        }
        for _ in 0..val_len {
            pop(&mut rt).unwrap();
        }
        assert_eq!(rt.operand_stack.len(), 0);

        ldc(&mut rt, Value::String("remember".into())).unwrap();
        ldc(&mut rt, Value::Unit).unwrap();
        pop(&mut rt).unwrap();
        assert_eq!(
            rt.operand_stack.pop().unwrap(),
            Value::String("remember".into())
        );

        let mut empty_rt = Runtime::new(vec![]);
        assert!(pop(&mut empty_rt).is_err());
    }
}
