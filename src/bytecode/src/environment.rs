use std::{cell::RefCell, collections::HashMap, rc::Rc};

use serde::{Deserialize, Deserializer, Serialize};

use crate::{Symbol, Value};

#[derive(Debug, Clone, Default)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    env: HashMap<Symbol, Value>,
}

impl Environment {
    /// Create a new frame with no parent, i.e. the root frame.
    pub fn new() -> Self {
        Environment {
            parent: None,
            env: HashMap::new(),
        }
    }

    /// Create a wrapped frame with no parent, i.e. the root frame.
    pub fn new_wrapped() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment::new()))
    }

    /// Set the parent of the frame.
    pub fn set_parent(&mut self, parent: Rc<RefCell<Environment>>) {
        self.parent = Some(parent);
    }
}

impl Environment {
    /// Get a snapshot of the value of a symbol in the frame at the time of the call.
    pub fn get(&self, sym: &Symbol) -> Option<Value> {
        if let Some(val) = self.env.get(sym) {
            Some(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(sym)
        } else {
            None
        }
    }

    /// Set the value of a symbol in the frame.
    pub fn set(&mut self, sym: impl Into<Symbol>, val: impl Into<Value>) {
        self.env.insert(sym.into(), val.into());
    }
}

/// Environment should NOT be serialized. It is only used for runtime state.
/// This trait is pseudo-implemented so that we can add it to the operant stack.
impl Serialize for Environment {
    fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        panic!("Environment should not be serialized");
    }
}

/// Environment should NOT be deserialized. It is only used for runtime state.
/// This trait is pseudo-implemented so that we can add it to the operant stack.
impl<'de> Deserialize<'de> for Environment {
    fn deserialize<D: Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        panic!("Environment should not be deserialized");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame() {
        let env = Environment::new_wrapped();
        env.borrow_mut().set("x", 42);
        assert_eq!(env.borrow().get(&"x".to_string()), Some(Value::Int(42)));
    }

    #[test]
    fn test_frame_with_parent() {
        let parent_env = Environment::new_wrapped();
        parent_env.borrow_mut().set("x", 42);
        let child_env = Environment::new_wrapped();
        child_env.borrow_mut().set_parent(parent_env);
        child_env.borrow_mut().set("y", 43);
        assert_eq!(
            child_env.borrow().get(&"x".to_string()),
            Some(Value::Int(42))
        );
        assert_eq!(
            child_env.borrow().get(&"y".to_string()),
            Some(Value::Int(43))
        );
    }
}
