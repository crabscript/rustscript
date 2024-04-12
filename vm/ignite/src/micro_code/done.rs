use anyhow::Result;

use crate::{Runtime, ThreadState};

/// Set the state of the current thread to done.
///
/// # Arguments
///
/// * `rt` - The runtime to set the current thread to done in.
///
/// # Errors
///
/// Infallible.
pub fn done(rt: &mut Runtime) -> Result<()> {
    rt.current_thread.state = ThreadState::Done;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_done() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        done(&mut rt)?;
        assert_eq!(rt.current_thread.state, ThreadState::Done);
        Ok(())
    }
}
