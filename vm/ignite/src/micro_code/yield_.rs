use std::time::Instant;

use anyhow::Result;

use crate::{Runtime, ThreadState, VmError};

/// Yield the current thread.
/// This will set the yield flag of the current thread to true.
/// The thread will be added to the ready queue on the next cycle of the VM.
///
/// # Arguments
///
/// * `rt` - The runtime to yield the current thread in.
///
/// # Errors
///
/// Infallible.
pub fn yield_(mut rt: Runtime) -> Result<Runtime> {
    let current_thread_id = rt.current_thread.thread_id;
    rt.set_thread_state(current_thread_id, ThreadState::Ready);

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
