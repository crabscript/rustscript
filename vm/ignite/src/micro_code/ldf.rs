use std::rc::Rc;

use anyhow::Result;
use bytecode::{FnType, Symbol, Value};

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
        fn_type: FnType::User,
        sym: "Closure".to_string(),
        prms,
        addr,
        env: Rc::clone(&rt.current_thread.env),
    };

    rt.current_thread.operand_stack.push(closure);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ldf() {
        let mut rt = Runtime::new(vec![]);
        ldf(&mut rt, 0, vec!["x".to_string()]).unwrap();

        let closure = rt.current_thread.operand_stack.pop().unwrap();
        assert_ne!(
            &closure,
            &Value::Closure {
                fn_type: FnType::User,
                sym: "Closure".to_string(),
                prms: vec!["y".to_string()],
                addr: 0,
                env: Rc::clone(&rt.current_thread.env),
            }
        )
    }
}
