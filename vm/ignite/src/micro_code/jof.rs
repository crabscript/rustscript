use anyhow::Result;
use bytecode::Value;

use crate::{Thread, VmError};

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
pub fn jof(t: &mut Thread, pc: usize) -> Result<()> {
    let cond = t
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;

    if let Value::Bool(b) = cond {
        if !b {
            t.pc = pc;
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
    use crate::Thread;

    #[test]
    fn test_jof() {
        let mut t = Thread::new(vec![]);
        ldc(&mut t, Value::Bool(false)).unwrap();
        jof(&mut t, 123).unwrap();
        assert_eq!(t.pc, 123);

        let mut rt = Thread::new(vec![]);
        ldc(&mut rt, Value::Bool(true)).unwrap();
        jof(&mut rt, 42).unwrap();
        assert_eq!(rt.pc, 0);

        ldc(&mut rt, Value::Unit).unwrap();
        let result = jof(&mut rt, 42);
        assert!(result.is_err());
    }
}
