use anyhow::{Ok, Result};

use crate::{Runtime, VmError};

use super::yield_;

/// Pop the operand stack for the thread ID to join.
/// If the thread to join is in zombie state, then the current thread will be set to ready and the result
/// of the zombie thread will be pushed onto the current thread's operand stack. The zombie thread is deallocated.
/// If the thread to join is not found, then panic.
/// Otherwise, the current thread will yield.
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
#[inline]
pub fn join(mut rt: Runtime) -> Result<Runtime> {
    let tid: i64 = rt
        .current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?
        .clone()
        .try_into()?;

    let Some(mut zombie_thread) = rt.zombie_threads.remove(&tid) else {
        // If the thread to join is not found, we need to yield control and try again
        rt.current_thread.pc -= 1; // Decrement the program counter to re-execute the join instruction
        rt.current_thread.operand_stack.push(tid.into()); // Add the pid back to the operand stack
        let rt = yield_(rt)?;
        return Ok(rt);
    };

    let result = zombie_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;

    // Deallocate the zombie thread
    drop(zombie_thread);

    rt.current_thread.operand_stack.push(result);
    Ok(rt)
}

#[cfg(test)]
mod tests {
    use bytecode::Value;

    use crate::{
        micro_code::{done, spawn},
        MAIN_THREAD_ID,
    };

    use super::*;

    #[test]
    fn test_join_01() -> Result<()> {
        let mut rt = Runtime::default();
        rt.current_thread.pc = 1; // prevent u64 subtraction overflow
        rt = spawn(rt, 0)?;
        rt = join(rt)?;
        // Add this point, both threads are in the ready state, so join should yield the current thread
        assert_eq!(rt.current_thread.thread_id, MAIN_THREAD_ID + 1);

        // Add the parent thread ID to the operand stack of the child
        rt = yield_(rt)?;
        assert_eq!(rt.current_thread.thread_id, MAIN_THREAD_ID);

        // PID should remain on the operand stack
        assert_eq!(rt.current_thread.operand_stack.len(), 1);

        Ok(())
    }

    #[test]
    fn test_join_02() -> Result<()> {
        let mut rt = Runtime::default();
        rt.current_thread.pc = 1; // prevent u64 subtraction overflow
        rt = spawn(rt, 0)?;
        rt = yield_(rt)?; // Yield the parent thread to make the child thread the current thread
        rt = done(rt)?; // Set the current thread to zombie state
        rt = yield_(rt)?; // Yield the child thread to make the parent thread the current thread

        rt = join(rt)?;
        // Add this point, the thread to join is in zombie state, so the current thread should just continue
        assert_eq!(rt.current_thread.thread_id, MAIN_THREAD_ID);
        // Zombie thread should be deallocated
        assert!(rt.zombie_threads.is_empty());
        // And the result of the zombie thread should be pushed onto the current thread's operand stack
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Int(0) // SPAWN adds 0 to the operand stack for the new thread
        );

        // The PID of child should be popped off the operand stack
        assert!(rt.current_thread.operand_stack.is_empty());

        Ok(())
    }
}
