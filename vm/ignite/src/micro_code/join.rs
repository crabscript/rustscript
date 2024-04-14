use std::collections::hash_map::Entry;

use anyhow::Result;

use crate::{Runtime, ThreadState, VmError};

/// Set the state of the current thread to joining the thread with the given ID from the operand stack.
/// The value on the operand stack is expected to be an integer and it is NOT popped off the stack.
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
/// * If the operand stack is empty.
/// * If the value on the operand stack is not an integer.
pub fn join(mut rt: Runtime) -> Result<Runtime> {
    let tid: i64 = rt
        .current_thread
        .operand_stack
        .last()
        .ok_or(VmError::OperandStackUnderflow)?
        .clone()
        .try_into()?;

    let current_tid = rt.current_thread.thread_id;
    let entry = rt.thread_states.entry(current_tid);

    match entry {
        Entry::Vacant(_) => Err(VmError::ThreadNotFound(current_tid).into()),
        Entry::Occupied(mut entry) => {
            entry.insert(ThreadState::Joining(tid));
            Ok(rt)
        }
    }
}

#[cfg(test)]
mod tests {
    use bytecode::Value;

    use super::*;

    #[test]
    fn test_join() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        rt.current_thread.operand_stack.push(Value::Int(1));
        rt = join(rt)?;
        assert_eq!(rt.thread_states.get(&1), Some(&ThreadState::Joining(1)));
        Ok(())
    }
}
