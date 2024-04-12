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
pub fn goto(rt: &mut Runtime, pc: usize) -> Result<()> {
    rt.current_thread.pc = pc;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goto() {
        let mut rt = Runtime::new(vec![]);
        goto(&mut rt, 123).unwrap();
        assert_eq!(rt.current_thread.pc, 123);
    }
}
