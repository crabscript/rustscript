use anyhow::{Ok, Result};
use bytecode::Symbol;

use crate::{Runtime, VmError};

/// Assign a value to a symbol.
///
/// # Arguments
///
/// * `rt` - The runtime to execute the instruction on.
///
/// * `sym` - The symbol to assign the value to.
///
/// # Errors
///
/// If the stack is empty.
use std::rc::Rc;
pub fn assign(rt: &mut Runtime, sym: Symbol) -> Result<()> {
    let val = rt
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;

    // to handle e.g let x = 2; { x = 10; } x
    // when the variable isnt in the current env we need to set the outer env's binding

    let mut env_ptr = Rc::clone(&rt.env);
    loop {
        if env_ptr.borrow().env.contains_key(&sym) || env_ptr.borrow().parent.is_none() {
            break;
        }

        let t = Rc::clone(env_ptr.borrow().parent.as_ref().unwrap());
        env_ptr = t;
    }

    // parent
    // if env_ptr.borrow().parent.is_none() && !env_ptr.borrow().env.contains_key(&sym){
    //     return Err(VmError::SymbolNotFound(sym.to_string()).into());
    // }

    env_ptr.borrow_mut().set(sym, val);

    Ok(())
}

#[cfg(test)]
mod tests {
    use bytecode::{Environment, Value};

    use super::*;

    #[test]
    fn test_assign() {
        let mut rt = Runtime::new(vec![]);
        rt.operand_stack.push(Value::Int(42));
        assign(&mut rt, "x".to_string()).unwrap();
        assert_eq!(rt.env.borrow().get(&"x".to_string()), Some(Value::Int(42)));
    }

    #[test]
    fn test_assign_with_parent() {
        let parent = Environment::new_wrapped();
        parent.borrow_mut().set("x", 42);
        let mut rt = Runtime::new(vec![]);
        let frame = Environment::new_wrapped();
        frame.borrow_mut().set_parent(parent);
        rt.env = frame;
        rt.operand_stack.push(Value::Int(43));
        assign(&mut rt, "y".to_string()).unwrap();
        assert_eq!(rt.env.borrow().get(&"x".to_string()), Some(Value::Int(42)));
        assert_eq!(rt.env.borrow().get(&"y".to_string()), Some(Value::Int(43)));
    }
}
