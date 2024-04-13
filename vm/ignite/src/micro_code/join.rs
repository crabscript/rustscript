use std::collections::hash_map::Entry;

use anyhow::Result;
use bytecode::ThreadID;

use crate::{Runtime, ThreadState, VmError};

/// Set the state of the current thread to joining the thread with the given ID.
///
/// # Arguments
///
/// * `rt` - The runtime to set the current thread to joining in.
///
/// * `tid` - The ID of the thread to join.
///
/// # Errors
///
/// * If the thread with the given ID is not found in the thread state hashmap.
pub fn join(rt: &mut Runtime, tid: ThreadID) -> Result<()> {
    let current_tid = rt.current_thread.thread_id;
    let entry = rt.thread_states.entry(current_tid);

    match entry {
        Entry::Vacant(_) => Err(VmError::ThreadNotFound(current_tid).into()),
        Entry::Occupied(mut entry) => {
            entry.insert(ThreadState::Joining(tid));
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        join(&mut rt, 2)?;
        assert_eq!(rt.thread_states.get(&1), Some(&ThreadState::Joining(2)));
        Ok(())
    }
}
