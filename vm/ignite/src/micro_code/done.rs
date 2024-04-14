use anyhow::{Ok, Result};

use crate::{Runtime, VmError, MAIN_THREAD_ID};

/// Set the state of the current thread to done.
///
/// # Arguments
///
/// * `rt` - The runtime to set the current thread to done in.
///
/// # Errors
///
/// * If the current thread is not found in the thread state hashmap.
pub fn done(mut rt: Runtime) -> Result<Runtime> {
    // If the current thread is the main thread, then we are done
    if rt.current_thread.thread_id == MAIN_THREAD_ID {
        rt.is_done = true;
        Ok(rt)
    // Otherwise we will set the current thread to zombie and yield
    } else {
        let current_thread = rt.current_thread;
        let current_thread_id = current_thread.thread_id;
        rt.zombie_threads.insert(current_thread_id, current_thread);

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
    use crate::micro_code::{spawn, yield_};

    use super::*;

    #[test]
    fn test_done_01() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        rt = done(rt)?;

        // The main thread should be done
        assert!(rt.is_done);

        Ok(())
    }

    #[test]
    fn test_done_02() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        rt = spawn(rt, 0)?;
        rt = yield_(rt)?; // Yield the control to the child thread
        rt = done(rt)?;

        // The main thread should not be done
        assert!(!rt.is_done);
        // The child thread should be in the zombie threads
        let child_thread_id = MAIN_THREAD_ID + 1;
        assert!(rt.zombie_threads.contains_key(&child_thread_id));
        // The current thread should be the main thread
        assert_eq!(rt.current_thread.thread_id, MAIN_THREAD_ID);

        Ok(())
    }
}
