use std::collections::hash_map::Entry;

use anyhow::{Ok, Result};
use bytecode::Semaphore;

use crate::{Runtime, ThreadState, VmError};

/// Pops a value off the stack.
/// The value is expected to be a semaphore.
/// If the semaphore is 0, the current thread is blocked.
/// If the semaphore is greater than 0, the semaphore is decremented.
///
/// # Arguments
///
/// * `rt` - The runtime to pop the value off of.
///
/// # Errors
///
/// If the stack is empty.
/// If the top value on stack is not a semaphore.
/// If the current thread is not found in the thread states.
pub fn wait(mut rt: Runtime) -> Result<Runtime> {
    let val = rt
        .current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;

    let current_tid = rt.current_thread.thread_id;
    let entry = rt.thread_states.entry(current_tid);

    let Entry::Occupied(mut entry) = entry else {
        return Err(VmError::ThreadNotFound(current_tid).into());
    };

    let sem: Semaphore = val.try_into()?;
    let mut sem_guard = sem.lock().unwrap();

    if *sem_guard == 0 {
        entry.insert(ThreadState::Blocked(sem.clone()));
    } else {
        *sem_guard -= 1;
    }

    Ok(rt)
}
