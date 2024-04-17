use std::rc::Weak;

use anyhow::Result;

use crate::{FnType, Value, W};
pub const INT_TO_FLOAT_SYM: &str = "int_to_float";

pub fn int_to_float() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: INT_TO_FLOAT_SYM.into(),
        prms: vec!["x".into()],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn int_to_float_impl(x: &Value) -> Result<Value> {
    let x: i64 = x.clone().try_into()?;
    Ok(Value::Float(x as f64))
}
