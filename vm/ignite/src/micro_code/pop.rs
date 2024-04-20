use anyhow::Result;

use crate::{Runtime, VmError};

/// Pops a value off the stack.
///
/// # Arguments
///
/// * `rt` - The runtime to pop the value off of.
///
/// # Errors
///
/// If the stack is empty.
#[inline]
pub fn pop(mut rt: Runtime) -> Result<Runtime> {
    rt.current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;
    Ok(rt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode::Value;

    use crate::micro_code::ldc;

    #[test]
    fn test_pop() {
        let mut rt = Runtime::new(vec![]);
        rt = ldc(rt, Value::Unit).unwrap();
        rt = pop(rt).unwrap();
        assert_eq!(rt.current_thread.operand_stack.len(), 0);

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
            rt = ldc(rt, val).unwrap();
        }
        for _ in 0..val_len {
            rt = pop(rt).unwrap();
        }
        assert_eq!(rt.current_thread.operand_stack.len(), 0);

        rt = ldc(rt, Value::String("remember".into())).unwrap();
        rt = ldc(rt, Value::Unit).unwrap();
        rt = pop(rt).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::String("remember".into())
        );

        let empty_rt = Runtime::new(vec![]);
        assert!(pop(empty_rt).is_err());
    }
}
