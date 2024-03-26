use std::rc::Rc;

use anyhow::Result;
use bytecode::{Symbol, Value, W};

use crate::Runtime;

/// Load a closure object onto the operand stack.
///
/// # Arguments
///
/// * `rt` - The runtime to load the closure onto.
///
/// * `addr` - The address of the closure.
///
/// * `prms` - The parameters of the closure.
///
/// # Errors
///
/// Infallible.
pub fn ldf(rt: &mut Runtime, addr: usize, prms: Vec<Symbol>) -> Result<()> {
    let closure = Value::Closure {
        sym: "Closure".to_string(),
        prms,
        addr,
        env: W(Rc::clone(&rt.env)),
    };

    rt.operand_stack.push(closure);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ldf() {
        let mut rt = Runtime::new(vec![]);
        ldf(&mut rt, 0, vec!["x".to_string()]).unwrap();

        let closure = rt.operand_stack.pop().unwrap();
        assert_eq!(
            &closure,
            &Value::Closure {
                sym: "Closure".to_string(),
                prms: vec!["x".to_string()],
                addr: 0,
                env: W(Rc::clone(&rt.env)),
            }
        );

        assert_ne!(
            &closure,
            &Value::Closure {
                sym: "Closure".to_string(),
                prms: vec!["y".to_string()],
                addr: 0,
                env: W(Rc::clone(&rt.env)),
            }
        )
    }
}
