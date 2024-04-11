use crate::Thread;

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
pub fn goto(t: &mut Thread, pc: usize) -> Result<()> {
    t.pc = pc;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Thread;

    #[test]
    fn test_goto() {
        let mut t = Thread::new(vec![]);
        goto(&mut t, 123).unwrap();
        assert_eq!(t.pc, 123);
    }
}
