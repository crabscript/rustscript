use std::{cell::RefCell, rc::Weak};

use anyhow::Result;
use bytecode::{weak_clone, Environment, StackFrame, Symbol, ThreadID, Value, W};

use crate::{Runtime, VmError};

/// A thread of execution.
/// Each thread has its own environment, operand stack, runtime stack, and program counter.
#[derive(Debug, Default, Clone)]
pub struct Thread {
    pub thread_id: ThreadID,
    pub env: Weak<RefCell<Environment>>,
    pub operand_stack: Vec<Value>,
    pub runtime_stack: Vec<StackFrame>,
    pub pc: usize,
}

impl Thread {
    pub fn new(thread_id: i64, env: Weak<RefCell<Environment>>) -> Self {
        Thread {
            thread_id,
            env,
            operand_stack: Vec::new(),
            runtime_stack: Vec::new(),
            ..Default::default()
        }
    }

    /// Create a new thread with the same environment as the current thread.
    /// But operand stack and runtime stack are empty.
    pub fn spawn_child(&self, thread_id: i64, pc: usize) -> Self {
        Thread {
            thread_id,
            env: Weak::clone(&self.env),
            operand_stack: Vec::new(),
            runtime_stack: Vec::new(),
            pc,
        }
    }
}

pub fn extend_environment<S, V>(mut rt: Runtime, syms: Vec<S>, vals: Vec<V>) -> Result<Runtime>
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

    let current_env = &rt
        .current_thread
        .env
        .upgrade()
        .ok_or(VmError::EnvironmentDroppedError)?;
    let new_env = Environment::new_wrapped();
    new_env.borrow_mut().set_parent(weak_clone(current_env));

    for (sym, val) in syms.into_iter().zip(vals.into_iter()) {
        new_env.borrow_mut().set(sym, val);
    }

    rt.current_thread.env = weak_clone(&new_env);
    rt.env_registry.insert(W(new_env));

    Ok(rt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode::Value;

    #[test]
    fn test_extend_environment_err() -> Result<()> {
        let mut rt = Runtime::default();
        let env = Environment::new_wrapped();
        rt.current_thread.env = weak_clone(&env);

        rt.current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow_mut()
            .set("a", 42);
        rt.current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow_mut()
            .set("b", 123);

        let empty: Vec<String> = Vec::new();
        let result = extend_environment(rt, vec!["c", "d"], empty);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_extend_environment() -> Result<()> {
        let mut rt = Runtime::default();
        let env = Environment::new_wrapped();
        rt.current_thread.env = weak_clone(&env);

        rt.current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow_mut()
            .set("a", 42);
        rt.current_thread
            .env
            .upgrade()
            .unwrap()
            .borrow_mut()
            .set("b", 123);

        let rt = extend_environment(
            rt,
            vec!["c", "d"],
            vec![Value::Float(12.3), Value::Bool(true)],
        )?;

        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"a".to_string())?,
            Value::Int(42)
        );

        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"b".to_string())?,
            Value::Int(123)
        );

        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"c".to_string())?,
            Value::Float(12.3)
        );

        assert_eq!(
            rt.current_thread
                .env
                .upgrade()
                .unwrap()
                .borrow()
                .get(&"d".to_string())?,
            Value::Bool(true)
        );

        Ok(())
    }
}
