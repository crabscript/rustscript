use std::collections::hash_map::Entry;

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
pub fn yield_(rt: &mut Runtime) -> Result<()> {
    let tid = rt.current_thread.thread_id;
    let entry = rt.thread_states.entry(tid);

    match entry {
        Entry::Vacant(_) => Err(VmError::ThreadNotFound(tid).into()),
        Entry::Occupied(mut entry) => {
            entry.insert(ThreadState::Yielded);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yield() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        yield_(&mut rt)?;
        assert_eq!(rt.thread_states.get(&1), Some(&ThreadState::Yielded));
        Ok(())
    }
}
