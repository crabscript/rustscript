use anyhow::Result;

use crate::Runtime;

/// Spawn a new thread that clones the main thread at the time of the spawn.
/// The new thread is added to the ready queue.
/// The new thread is given a unique thread ID.
/// This thread ID is pushed onto the operand stack of the parent thread.
/// 0 is pushed onto the operand stack of the child thread.
///
/// # Arguments
///
/// * `rt` - The runtime to spawn a new thread in.
///
/// # Errors
///
/// Infallible.
pub fn spawn(rt: &mut Runtime) -> Result<()> {
    rt.thread_count += 1;

    let new_thread_id = rt.thread_count;
    let mut new_thread = rt.current_thread.spawn_new(new_thread_id);

    // The child thread ID is pushed onto the operand stack of the parent thread.
    rt.current_thread.operand_stack.push(new_thread_id.into());
    // 0 is pushed onto the operand stack of the child thread.
    new_thread.operand_stack.push(0.into());

    rt.ready_queue.push_back(new_thread);
    Ok(())
}
