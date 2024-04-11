use anyhow::Result;

use crate::{Thread, VmError};

/// Pops a value off the stack.
///
/// # Arguments
///
/// * `rt` - The runtime to pop the value off of.
///
/// # Errors
///
/// If the stack is empty.
pub fn pop(t: &mut Thread) -> Result<()> {
    t.operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode::Value;

    use crate::micro_code::ldc;
    use crate::Thread;

    #[test]
    fn test_pop() {
        let mut t = Thread::new(vec![]);
        ldc(&mut t, Value::Unit).unwrap();
        pop(&mut t).unwrap();
        assert_eq!(t.operand_stack.len(), 0);

        let vals = vec![
            Value::Unit,
            Value::Int(42),
            Value::Float(42.0),
            Value::Bool(true),
            Value::String("hello world".into()),
        ];
        let val_len = vals.len();
        let mut rt = Thread::new(vec![]);
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

        let mut empty_rt = Thread::new(vec![]);
        assert!(pop(&mut empty_rt).is_err());
    }
}
