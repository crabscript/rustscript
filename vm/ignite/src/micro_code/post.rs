use anyhow::{Ok, Result};
use bytecode::Semaphore;

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
pub fn post(rt: &mut Runtime) -> Result<()> {
    let val = rt
        .current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;

    let sem: Semaphore = val.try_into()?;
    rt.signal = Some(sem.clone());

    Ok(())
}
