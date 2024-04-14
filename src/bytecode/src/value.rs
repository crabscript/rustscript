use std::{cell::RefCell, fmt::Display, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{ByteCodeError, Environment, Semaphore, Symbol};

/// The values that can be stored on the operant stack.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Value {
    Unitialized,
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    #[serde(skip_serializing, skip_deserializing)]
    Semaphore(Semaphore),
    #[serde(skip_serializing, skip_deserializing)]
    Closure {
        fn_type: FnType,
        sym: Symbol,
        prms: Vec<Symbol>,
        addr: usize,
        env: Rc<RefCell<Environment>>,
    },
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum FnType {
    #[default]
    User,
    Builtin,
}

pub fn type_of(value: &Value) -> &'static str {
    match value {
        Value::Unitialized => "Unitialized",
        Value::Unit => "Unit",
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::Bool(_) => "Bool",
        Value::String(_) => "String",
        Value::Semaphore(_) => "Semaphore",
        Value::Closure { .. } => "Closure",
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Value::Unitialized => "uninitialized".to_string(),
            Value::Unit => "()".to_string(),
            Value::String(s) => s.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Semaphore(_) => "semaphore".to_string(),
            Value::Closure { .. } => "closure".to_string(),
        };

        write!(f, "{}", res)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float(v)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Unit
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}

impl From<Semaphore> for Value {
    fn from(v: Semaphore) -> Self {
        Value::Semaphore(v)
    }
}

impl TryFrom<Value> for () {
    type Error = ByteCodeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Unit => Ok(()),
            _ => Err(ByteCodeError::TypeMismatch {
                expected: "Unit".to_string(),
                found: format!("{:?}", value),
            }),
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = ByteCodeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int(i) => Ok(i),
            _ => Err(ByteCodeError::TypeMismatch {
                expected: "Int".to_string(),
                found: format!("{:?}", value),
            }),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = ByteCodeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Float(f) => Ok(f),
            _ => Err(ByteCodeError::TypeMismatch {
                expected: "Float".to_string(),
                found: format!("{:?}", value),
            }),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = ByteCodeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(b) => Ok(b),
            _ => Err(ByteCodeError::TypeMismatch {
                expected: "Bool".to_string(),
                found: format!("{:?}", value),
            }),
        }
    }
}

impl TryFrom<Value> for String {
    type Error = ByteCodeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(ByteCodeError::TypeMismatch {
                expected: "String".to_string(),
                found: format!("{:?}", value),
            }),
        }
    }
}

impl TryFrom<Value> for Semaphore {
    type Error = ByteCodeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Semaphore(s) => Ok(s),
            _ => Err(ByteCodeError::TypeMismatch {
                expected: "Semaphore".to_string(),
                found: format!("{:?}", value),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_from_i64() {
        let v: Value = 42.into();
        assert_eq!(v, Value::Int(42));

        let v: Value = 0.into();
        assert_eq!(v, Value::Int(0));
    }

    #[test]
    fn test_value_from_f64() {
        let v: Value = 42.0.into();
        assert_eq!(v, Value::Float(42.0));

        let v: Value = 0.0.into();
        assert_eq!(v, Value::Float(0.0));
    }

    #[test]
    fn test_unequal() {
        let val_int: Value = 42.into();
        let val_float: Value = 42.0.into();
        let val_bool: Value = true.into();
        let val_string: Value = "42".into();

        assert_ne!(val_int, val_float);
        assert_ne!(val_int, val_bool);
        assert_ne!(val_float, val_bool);
        assert_ne!(val_int, val_string);
    }

    #[test]
    fn test_from_bool() {
        let bool_value: bool = true;
        let value: Value = bool_value.into();
        assert_eq!(value, Value::Bool(bool_value));

        let bool_value: bool = false;
        let value: Value = bool_value.into();
        assert_eq!(value, Value::Bool(bool_value));
    }

    #[test]
    fn test_from_unit() {
        let value: Value = ().into();
        assert_eq!(value, Value::Unit);
    }

    #[test]
    fn test_from_string() {
        let string_value: String = "Hello, World!".to_string();
        let value: Value = string_value.clone().into();
        assert_eq!(value, Value::String(string_value));
    }
}
