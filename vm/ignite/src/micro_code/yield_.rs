use anyhow::Result;

use crate::Runtime;

pub fn yield_(rt: &mut Runtime) -> Result<()> {
    // Move the current thread to the back of the ready queue
    // rt.ready_queue.push_back(rt.current_thread);

    let next_thread = rt.ready_queue.pop_front().expect("No threads ready to run");
    Ok(())
}
