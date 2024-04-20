use anyhow::Result;

use crate::Runtime;

/// Spawn a child thread that clones the current/parent thread at the time of the spawn.
/// The child thread is given a unique thread ID.
/// The child thread is added to the back of the ready queue.
/// This thread ID is pushed onto the operand stack of the parent thread.
/// 0 is pushed onto the operand stack of the child thread.
/// The child thread starts execution at the given address.
/// The parent thread continues execution.
///
/// # Arguments
///
/// * `rt` - The runtime to spawn a new thread in.
///
/// # Errors
///
/// Infallible.
#[inline]
pub fn spawn(mut rt: Runtime, addr: usize) -> Result<Runtime> {
    rt.thread_count += 1;

    let child_thread_id = rt.thread_count;
    let mut child_thread = rt.current_thread.spawn_child(child_thread_id, addr);

    // 0 is pushed onto the operand stack of the child thread.
    child_thread.operand_stack.push(0.into());
    // The child thread ID is pushed onto the operand stack of the parent thread.
    rt.current_thread.operand_stack.push(child_thread_id.into());

    rt.ready_queue.push_back(child_thread);
    Ok(rt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn() -> Result<()> {
        let rt = Runtime::new(vec![]);
        let rt = spawn(rt, 0)?;
        assert_eq!(rt.thread_count, 2);
        assert_eq!(rt.ready_queue.len(), 1);
        Ok(())
    }
}
