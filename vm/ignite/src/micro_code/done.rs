use std::collections::hash_map::Entry;

use anyhow::{Ok, Result};

use crate::{Runtime, ThreadState, VmError};

/// Set the state of the current thread to done.
///
/// # Arguments
///
/// * `rt` - The runtime to set the current thread to done in.
///
/// # Errors
///
/// * If the current thread is not found in the thread state hashmap.
pub fn done(rt: &mut Runtime) -> Result<()> {
    let tid = rt.current_thread.thread_id;
    let entry = rt.thread_states.entry(tid);

    match entry {
        Entry::Vacant(_) => Err(VmError::ThreadNotFound(tid).into()),
        Entry::Occupied(mut entry) => {
            entry.insert(ThreadState::Done);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_done() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        done(&mut rt)?;
        assert_eq!(rt.thread_states.get(&1), Some(&ThreadState::Done));
        Ok(())
    }
}
