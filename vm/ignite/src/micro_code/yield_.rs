use anyhow::Result;

use crate::{Runtime, ThreadState};

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
    rt.current_thread.state = ThreadState::Yielded;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yield() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        yield_(&mut rt)?;
        assert_eq!(rt.current_thread.state, ThreadState::Yielded);
        Ok(())
    }
}
