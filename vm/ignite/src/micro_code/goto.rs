use crate::Runtime;

use anyhow::Result;

/// Jumps to the given program counter.
///
/// # Arguments
///
/// * `rt` - The runtime to execute the operation on.
///
/// * `pc` - The program counter to jump to.
///
/// # Errors
///
/// Infallible.
#[inline]
pub fn goto(mut rt: Runtime, pc: usize) -> Result<Runtime> {
    rt.current_thread.pc = pc;
    Ok(rt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goto() {
        let mut rt = Runtime::new(vec![]);
        rt = goto(rt, 123).unwrap();
        assert_eq!(rt.current_thread.pc, 123);
    }
}
