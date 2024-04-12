use anyhow::Result;
use bytecode::Value;

use crate::{Runtime, VmError};

/// Jumps to the given program counter if the top of the stack is false.
///
/// # Arguments
///
/// * `rt` - The runtime to execute the operation on.
///
/// * `pc` - The program counter to jump to.
///
/// # Errors
///
/// If the stack is empty or the top of the stack is not a boolean.
pub fn jof(rt: &mut Runtime, pc: usize) -> Result<()> {
    let cond = rt
        .current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;

    if let Value::Bool(b) = cond {
        if !b {
            rt.current_thread.pc = pc;
        }

        Ok(())
    } else {
        Err(VmError::IllegalArgument("expected bool".to_string()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode::Value;

    use crate::micro_code::ldc;

    #[test]
    fn test_jof() {
        let mut rt = Runtime::new(vec![]);
        ldc(&mut rt, Value::Bool(false)).unwrap();
        jof(&mut rt, 123).unwrap();
        assert_eq!(rt.current_thread.pc, 123);

        let mut rt = Runtime::new(vec![]);
        ldc(&mut rt, Value::Bool(true)).unwrap();
        jof(&mut rt, 42).unwrap();
        assert_eq!(rt.current_thread.pc, 0);

        ldc(&mut rt, Value::Unit).unwrap();
        let result = jof(&mut rt, 42);
        assert!(result.is_err());
    }
}
