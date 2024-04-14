use anyhow::{Ok, Result};
use bytecode::Semaphore;

use crate::{Runtime, VmError};

/// Pops a value off the stack.
/// The value is expected to be a semaphore.
/// The semaphore is used to signal to the runtime that any threads waiting on it can continue.
///
/// # Arguments
///
/// * `rt` - The runtime to pop the value off of.
///
/// # Errors
///
/// If the stack is empty.
/// If the top value on stack is not a semaphore.
pub fn post(mut rt: Runtime) -> Result<Runtime> {
    let val = rt
        .current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;

    let sem: Semaphore = val.try_into()?;
    rt.signal = Some(sem.clone());

    Ok(rt)
}
