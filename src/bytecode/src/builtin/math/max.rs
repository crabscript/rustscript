use std::rc::Weak;

use anyhow::Result;

use crate::{FnType, Value, W};

pub const MAX_SYM: &str = "max";

pub fn max() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: MAX_SYM.into(),
        prms: vec!["v1".into(), "v2".into()],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn max_impl(v1: &Value, v2: &Value) -> Result<Value> {
    match (v1.clone(), v2.clone()) {
        (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1.max(v2))),
        (Value::Float(v1), Value::Float(v2)) => Ok(Value::Float(v1.max(v2))),
        _ => Err(crate::ByteCodeError::TypeMismatch {
            expected: crate::type_of(v1).to_string(),
            found: crate::type_of(v2).to_string(),
        }
        .into()),
    }
}
