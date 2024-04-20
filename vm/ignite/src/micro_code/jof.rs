use anyhow::Result;

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
#[inline]
pub fn jof(mut rt: Runtime, pc: usize) -> Result<Runtime> {
    let cond = rt
        .current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;

    let b: bool = cond.try_into()?;
    if !b {
        rt.current_thread.pc = pc;
    }

    Ok(rt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode::Value;

    use crate::micro_code::ldc;

    #[test]
    fn test_jof() {
        let mut rt = Runtime::new(vec![]);
        rt = ldc(rt, Value::Bool(false)).unwrap();
        rt = jof(rt, 123).unwrap();
        assert_eq!(rt.current_thread.pc, 123);

        let mut rt = Runtime::new(vec![]);
        rt = ldc(rt, Value::Bool(true)).unwrap();
        rt = jof(rt, 42).unwrap();
        assert_eq!(rt.current_thread.pc, 0);

        rt = ldc(rt, Value::Unit).unwrap();
        let result = jof(rt, 42);
        assert!(result.is_err());
    }
}
