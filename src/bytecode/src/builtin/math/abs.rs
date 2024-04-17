use std::rc::Weak;

use anyhow::Result;

use crate::{type_of, ByteCodeError, FnType, Value, W};

pub const ABS_SYM: &str = "abs";

pub fn abs() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: ABS_SYM.into(),
        prms: vec!["x".into()],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn abs_impl(x: &Value) -> Result<Value> {
    match x.clone() {
        Value::Int(x) => Ok(Value::Int(x.abs())),
        Value::Float(x) => Ok(Value::Float(x.abs())),
        _ => Err(ByteCodeError::BadType {
            expected: "Integer or Float".to_string(),
            found: type_of(x).to_string(),
        }
        .into()),
    }
}
