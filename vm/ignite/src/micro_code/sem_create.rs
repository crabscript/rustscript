use anyhow::Result;
use bytecode::Semaphore;

use crate::Runtime;

/// Loads a semaphore onto the stack.
///
/// # Arguments
///
/// * `rt` - The runtime to load the semaphore onto.
///
/// # Errors
///
/// Infallible.
#[inline]
pub fn sem_create(mut rt: Runtime) -> Result<Runtime> {
    rt.current_thread
        .operand_stack
        .push(Semaphore::new(1).into());
    Ok(rt)
}

#[cfg(test)]
mod tests {
    use bytecode::type_of;

    use super::*;

    #[test]
    fn test_ldc() {
        let mut rt = Runtime::new(vec![]);
        rt = sem_create(rt).unwrap();
        assert_eq!(
            type_of(&rt.current_thread.operand_stack.pop().unwrap()),
            type_of(&Semaphore::new(1).into())
        );
    }
}
