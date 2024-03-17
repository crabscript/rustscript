use bytecode::{Symbol, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Frame {
    pub parent: Option<Box<Frame>>,
    pub env: HashMap<Symbol, Value>,
}

impl Frame {
    pub fn new(parent: Option<Box<Frame>>) -> Self {
        Frame {
            parent,
            env: HashMap::new(),
        }
    }

    pub fn get(&self, sym: &Symbol) -> Option<&Value> {
        if let Some(val) = self.env.get(sym) {
            Some(val)
        } else if let Some(parent) = &self.parent {
            parent.get(sym)
        } else {
            None
        }
    }

    pub fn set(&mut self, sym: impl Into<Symbol>, val: impl Into<Value>) {
        self.env.insert(sym.into(), val.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame() {
        let mut frame = Frame::new(None);
        frame.set("x", 42);
        assert_eq!(frame.get(&"x".to_string()), Some(&Value::Int(42)));
    }

    #[test]
    fn test_frame_with_parent() {
        let mut parent = Frame::new(None);
        parent.set("x", 42);
        let mut frame = Frame::new(Some(Box::new(parent)));
        frame.set("y", 43);
        assert_eq!(frame.get(&"x".to_string()), Some(&Value::Int(42)));
        assert_eq!(frame.get(&"y".to_string()), Some(&Value::Int(43)));
    }
}
