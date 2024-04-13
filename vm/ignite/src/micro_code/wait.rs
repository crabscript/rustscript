use std::collections::hash_map::Entry;

use anyhow::{Ok, Result};
use bytecode::Semaphore;

use crate::{Runtime, ThreadState, VmError};

pub fn wait(rt: &mut Runtime) -> Result<()> {
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

    Ok(())
}
