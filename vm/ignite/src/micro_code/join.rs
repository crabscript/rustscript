use anyhow::Result;
use bytecode::ThreadID;

use crate::{Runtime, ThreadState};

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
/// Infallible.
pub fn join(rt: &mut Runtime, tid: ThreadID) -> Result<()> {
    rt.current_thread.state = ThreadState::Joining(tid);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        join(&mut rt, 2)?;
        assert_eq!(rt.current_thread.state, ThreadState::Joining(2));
        Ok(())
    }
}
