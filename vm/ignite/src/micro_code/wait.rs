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
    let sem: Semaphore = rt
        .current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?
        .try_into()?;
    let mut sem_guard = sem.lock().unwrap();

    if *sem_guard > 0 {
        *sem_guard -= 1;
        drop(sem_guard); //unlock the semaphore

        Ok(rt)
    } else {
        drop(sem_guard); //unlock the semaphore

        let current_tid = rt.current_thread.thread_id;
        let entry = rt.thread_states.entry(current_tid);
        let Entry::Occupied(mut entry) = entry else {
            return Err(VmError::ThreadNotFound(current_tid).into());
        };
        entry.insert(ThreadState::Blocked(sem.clone()));

        // Move the current thread to the blocked queue and pop the next ready thread.
        let current_thread = rt.current_thread;
        rt.blocked_queue.push_back(current_thread);

        let next_ready_thread = rt
            .ready_queue
            .pop_front()
            .ok_or(VmError::NoThreadsInReadyQueue)?;

        rt.current_thread = next_ready_thread;
        Ok(rt)
    }
}

#[cfg(test)]
mod tests {
    use crate::{micro_code, MAIN_THREAD_ID};

    use super::*;

    #[test]
    fn test_wait_01() -> Result<()> {
        let mut rt = Runtime::default();
        let sem = Semaphore::new(1);
        rt.current_thread
            .extend_environment(vec!["sem"], vec![sem.clone()])?;
        rt = micro_code::spawn(rt, 0)?; // spawn a child thread to populate ready queue
        rt = micro_code::ld(rt, "sem".into())?; // load the semaphore onto the stack
        rt = wait(rt)?;

        assert_eq!(*sem.lock().unwrap(), 0);
        // Since the semaphore greater than 0, the semaphore should be decremented and the current thread should continue.
        assert_eq!(rt.current_thread.thread_id, MAIN_THREAD_ID);

        Ok(())
    }

    #[test]
    fn test_wait_02() -> Result<()> {
        let mut rt = Runtime::default();
        let sem = Semaphore::new(0);
        rt.current_thread
            .extend_environment(vec!["sem"], vec![sem.clone()])?;
        rt = micro_code::spawn(rt, 0)?; // spawn a child thread to populate ready queue
        rt = micro_code::ld(rt, "sem".into())?; // load the semaphore onto the stack
        rt = wait(rt)?;

        let child_thread_id = MAIN_THREAD_ID + 1;
        assert_eq!(*sem.lock().unwrap(), 0);
        // Since the semaphore is 0, the current thread should be blocked.
        assert_eq!(
            rt.blocked_queue.pop_front().unwrap().thread_id,
            MAIN_THREAD_ID
        );
        // The child thread should be the current thread.
        assert_eq!(rt.current_thread.thread_id, child_thread_id);
        // The state of the parent thread should be blocked.
        assert_eq!(
            rt.thread_states.get(&MAIN_THREAD_ID).unwrap(),
            &ThreadState::Blocked(sem.clone())
        );

        Ok(())
    }
}
