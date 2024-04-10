use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{type_of, ByteCodeError, Environment, FnType, Value};

pub const MIN_SYM: &str = "min";

pub fn min(global_env: Rc<RefCell<Environment>>) -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: MIN_SYM.into(),
        prms: vec!["v1".into(), "v2".into()],
        addr: 0,
        env: global_env,
    }
}

pub fn min_impl(v1: &Value, v2: &Value) -> Result<Value> {
    match (v1.clone(), v2.clone()) {
        (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1.min(v2))),
        (Value::Float(v1), Value::Float(v2)) => Ok(Value::Float(v1.min(v2))),
        _ => Err(ByteCodeError::TypeMismatch {
            expected: type_of(v1).to_string(),
            found: type_of(v2).to_string(),
        }
        .into()),
    }
}
