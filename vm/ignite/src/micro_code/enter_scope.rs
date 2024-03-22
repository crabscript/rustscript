use std::rc::Rc;

use crate::{runtime::extend_environment, Runtime, StackFrame};
use anyhow::Result;
use bytecode::{FrameType, Symbol, Value};

pub fn enter_scope(rt: &mut Runtime, syms: Vec<Symbol>) -> Result<()> {
    let current_env = Rc::clone(&rt.env);

    // Preserve the current environment in a stack frame
    let frame = StackFrame::new(FrameType::BlockFrame, Rc::clone(&current_env));

    // Push the stack frame onto the runtime stack
    rt.runtime_stack.push(frame);

    let uninitialized = syms
        .iter()
        .map(|_| Value::Unitialized)
        .collect::<Vec<Value>>();

    extend_environment(rt, syms, uninitialized)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode::Value;

    #[test]
    fn test_enter_scope() {
        let mut rt = Runtime::new(vec![]);
        let env = Rc::clone(&rt.env);

        rt.env.borrow_mut().set("a", 42);
        rt.env.borrow_mut().set("b", 123);

        enter_scope(&mut rt, vec!["c".to_string(), "d".to_string()]).unwrap();

        assert_eq!(rt.runtime_stack.len(), 1);
        assert_eq!(rt.env.borrow().get(&"a".to_string()), Some(Value::Int(42)));
        assert_eq!(rt.env.borrow().get(&"b".to_string()), Some(Value::Int(123)));
        assert_eq!(
            rt.env.borrow().get(&"c".to_string()),
            Some(Value::Unitialized)
        );
        assert_eq!(
            rt.env.borrow().get(&"d".to_string()),
            Some(Value::Unitialized)
        );

        rt.env = env;

        assert_eq!(rt.env.borrow().get(&"a".to_string()), Some(Value::Int(42)));
        assert_eq!(rt.env.borrow().get(&"b".to_string()), Some(Value::Int(123)));
        assert_eq!(rt.env.borrow().get(&"c".to_string()), None);
        assert_eq!(rt.env.borrow().get(&"d".to_string()), None);
    }
}
