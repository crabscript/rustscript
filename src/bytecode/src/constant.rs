use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
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

        assert_ne!(val_int, val_float);
        assert_ne!(val_int, val_bool);
        assert_ne!(val_float, val_bool);
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
}
