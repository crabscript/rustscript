use bytecode::{Symbol, Value};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Clone, Default)]
pub struct Frame {
    pub parent: Option<Rc<RefCell<Frame>>>,
    pub env: HashMap<Symbol, Value>,
}

impl Frame {
    /// Create a new frame with no parent, i.e. the root frame.
    pub fn new() -> Self {
        Frame {
            parent: None,
            env: HashMap::new(),
        }
    }

    /// Create a wrapped frame with no parent, i.e. the root frame.
    pub fn new_wrapped() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Frame::new()))
    }

    /// Set the parent of the frame.
    pub fn set_parent(&mut self, parent: Rc<RefCell<Frame>>) {
        self.parent = Some(parent);
    }
}

impl Frame {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame() {
        let frame = Frame::new_wrapped();
        frame.borrow_mut().set("x", 42);
        assert_eq!(frame.borrow().get(&"x".to_string()), Some(Value::Int(42)));
    }

    #[test]
    fn test_frame_with_parent() {
        let parent = Frame::new_wrapped();
        parent.borrow_mut().set("x", 42);
        let frame = Frame::new_wrapped();
        frame.borrow_mut().set_parent(parent);
        frame.borrow_mut().set("y", 43);
        assert_eq!(frame.borrow().get(&"x".to_string()), Some(Value::Int(42)));
        assert_eq!(frame.borrow().get(&"y".to_string()), Some(Value::Int(43)));
    }
}
