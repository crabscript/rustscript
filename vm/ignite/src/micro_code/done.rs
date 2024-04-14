use std::collections::hash_map::Entry;

use anyhow::{Ok, Result};

use crate::{Runtime, ThreadState, VmError, MAIN_THREAD_ID};

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
    let tid = rt.current_thread.thread_id;
    let entry = rt.thread_states.entry(tid);

    let Entry::Occupied(mut entry) = entry else {
        return Err(VmError::ThreadNotFound(tid).into());
    };

    // If the current thread is the main thread, then we are done
    if rt.current_thread.thread_id == MAIN_THREAD_ID {
        entry.insert(ThreadState::Done);
        Ok(rt)
    // Otherwise we will set the current thread to zombie and yield
    } else {
        entry.insert(ThreadState::Zombie);

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
    use crate::micro_code::spawn;

    use super::*;

    #[test]
    fn test_done() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        rt = done(rt)?;

        // The main thread should be done
        assert_eq!(rt.thread_states.get(&1), Some(&ThreadState::Done));

        let mut rt = Runtime::new(vec![]);
        rt = spawn(rt, 0)?;
        rt.current_thread.thread_id = 2;
        rt = done(rt)?;

        // The current thread should be zombie
        assert_eq!(rt.thread_states.get(&2), Some(&ThreadState::Zombie));

        Ok(())
    }
}
