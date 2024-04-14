use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use bytecode::{Environment, Semaphore, StackFrame, Symbol, ThreadID, Value};

use crate::VmError;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum ThreadState {
    #[default]
    Ready,
    Blocked(Semaphore),
    Zombie,
    Done,
}

/// A thread of execution.
#[derive(Debug, Default, Clone)]
pub struct Thread {
    /// The unique identifier of the thread.
    pub thread_id: ThreadID,
    pub env: Rc<RefCell<Environment>>,
    pub operand_stack: Vec<Value>,
    pub runtime_stack: Vec<StackFrame>,
    pub pc: usize,
}

impl Thread {
    pub fn new(thread_id: i64) -> Self {
        Thread {
            thread_id,
            env: Environment::new_global(),
            operand_stack: Vec::new(),
            runtime_stack: Vec::new(),
            ..Default::default()
        }
    }
}

impl Thread {
    pub fn spawn_new(&self, thread_id: i64, pc: usize) -> Thread {
        Thread {
            thread_id,
            env: Rc::clone(&self.env),
            operand_stack: Vec::new(),
            runtime_stack: Vec::new(),
            pc,
        }
    }
}

/// Extend the current environment with new symbols and values.
///
/// # Arguments
///
/// * `rt` - The runtime to extend the environment of.
///
/// * `syms` - The symbols to add to the environment.
///
/// * `vals` - The values to add to the environment.
///
/// # Errors
///
/// If the symbols and values are not the same length.
pub fn extend_environment<S, V>(t: &mut Thread, syms: Vec<S>, vals: Vec<V>) -> Result<()>
where
    S: Into<Symbol>,
    V: Into<Value>,
{
    if syms.len() != vals.len() {
        return Err(VmError::IllegalArgument(
            "symbols and values must be the same length".to_string(),
        )
        .into());
    }

    let current_env = Rc::clone(&t.env);
    let new_env = Environment::new_wrapped();
    new_env.borrow_mut().set_parent(current_env);

    for (sym, val) in syms.into_iter().zip(vals.into_iter()) {
        new_env.borrow_mut().set(sym, val);
    }

    t.env = new_env;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode::Value;

    #[test]
    fn test_extend_environment() -> Result<()> {
        let mut t = Thread::new(1);
        t.env.borrow_mut().set("a", 42);
        t.env.borrow_mut().set("b", 123);

        let empty: Vec<String> = Vec::new();
        assert!(extend_environment(&mut t, vec!["c", "d"], empty).is_err());

        extend_environment(
            &mut t,
            vec!["c", "d"],
            vec![Value::Float(12.3), Value::Bool(true)],
        )?;

        assert_eq!(t.env.borrow().get(&"a".to_string()), Some(Value::Int(42)));
        assert_eq!(t.env.borrow().get(&"b".to_string()), Some(Value::Int(123)));
        assert_eq!(
            t.env.borrow().get(&"c".to_string()),
            Some(Value::Float(12.3))
        );
        assert_eq!(
            t.env.borrow().get(&"d".to_string()),
            Some(Value::Bool(true))
        );

        Ok(())
    }
}
