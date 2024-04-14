use std::time::Instant;

use anyhow::Result;

use crate::{Runtime, VmError};

/// Yield the current thread in the runtime.
/// Push the current thread to the back of the ready queue.
/// Pop the next ready thread from the front of the ready queue and set it as the current thread.
///
/// # Arguments
///
/// * `rt` - The runtime to yield the current thread in.
///
/// # Errors
///
/// Returns an error if there are no threads in the ready queue.
pub fn yield_(mut rt: Runtime) -> Result<Runtime> {
    let current_thread = rt.current_thread;
    rt.ready_queue.push_back(current_thread);

    let next_ready_thread = rt
        .ready_queue
        .pop_front()
        .ok_or(VmError::NoThreadsInReadyQueue)?;

    rt.current_thread = next_ready_thread;
    rt.time = Instant::now(); // Reset the time
    Ok(rt)
}

#[cfg(test)]
mod tests {
    use crate::{micro_code::spawn, MAIN_THREAD_ID};

    use super::*;

    #[test]
    fn test_yield() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        rt = spawn(rt, 1)?;
        rt = yield_(rt)?;

        assert_eq!(rt.current_thread.thread_id, MAIN_THREAD_ID + 1);

        Ok(())
    }
}
