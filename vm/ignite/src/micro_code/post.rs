use anyhow::{Ok, Result};
use bytecode::Semaphore;

use crate::{Runtime, VmError};

/// Pops a value off the stack.
/// The value is expected to be a semaphore.
/// The semaphore is incremented.
/// If a thread is blocked on this semaphore, the first blocked thread is moved to the ready queue.
/// The current thread continues execution.
///
/// # Arguments
///
/// * `rt` - The runtime to pop the value off of.
///
/// # Errors
///
/// If the stack is empty.
/// If the top value on stack is not a semaphore.
#[inline]
pub fn post(mut rt: Runtime) -> Result<Runtime> {
    let sem: Semaphore = rt
        .current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?
        .try_into()?;

    let mut sem_guard = sem.lock().unwrap();
    *sem_guard += 1;

    // Find the first blocked thread that is waiting on the semaphore.
    let blocked_thread = rt
        .blocked_queue
        .iter()
        .position(|(_, blocking_sem)| blocking_sem == &sem)
        .map(|i| rt.blocked_queue.remove(i));

    let Some(Some((blocked_thread, _))) = blocked_thread else {
        // If no blocked threads are found, nothing needs to be done.
        return Ok(rt);
    };

    *sem_guard -= 1;
    drop(sem_guard); // Unlock the semaphore.

    // Move the blocked thread to the ready queue.
    rt.ready_queue.push_back(blocked_thread);
    Ok(rt)
}

#[cfg(test)]
mod tests {
    use crate::{
        extend_environment,
        micro_code::{ld, spawn, wait, yield_},
        MAIN_THREAD_ID,
    };

    use super::*;

    #[test]
    fn test_post_01() -> Result<()> {
        let mut rt = Runtime::default();
        let sem = Semaphore::new(0);
        let current_env = rt.current_thread.env.clone();
        rt = extend_environment(rt, current_env, vec!["sem"], vec![sem.clone()])?;
        rt = spawn(rt, 0)?; // spawn a child thread to populate ready queue
        rt = ld(rt, "sem".into())?;
        rt = post(rt)?;

        // Since no threads are blocked on the semaphore, the current thread should continue.
        assert_eq!(rt.current_thread.thread_id, MAIN_THREAD_ID);
        // The semaphore should be incremented.
        assert_eq!(*sem.lock().unwrap(), 1);

        Ok(())
    }

    #[test]
    fn test_post_02() -> Result<()> {
        let mut rt = Runtime::default();
        let sem = Semaphore::new(0);
        let current_env = rt.current_thread.env.clone();
        rt = extend_environment(rt, current_env, vec!["sem"], vec![sem.clone()])?;
        rt = spawn(rt, 0)?; // spawn a child thread to populate ready queue
        rt = yield_(rt)?; // yield the current thread to child thread
        rt = ld(rt, "sem".into())?;
        rt = wait(rt)?;
        rt = ld(rt, "sem".into())?;
        rt = post(rt)?;

        // Child thread should be moved to the ready queue.
        let child_thread_id = MAIN_THREAD_ID + 1;
        assert_eq!(
            rt.ready_queue.pop_front().unwrap().thread_id,
            child_thread_id
        );

        Ok(())
    }
}
