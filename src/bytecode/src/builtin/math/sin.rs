use std::rc::Weak;

use anyhow::Result;

use crate::{FnType, Value, W};

pub const SIN_SYM: &str = "sin";

pub fn sin() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: SIN_SYM.into(),
        prms: vec!["x".into()],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn sin_impl(x: &Value) -> Result<Value> {
    let x: f64 = x.clone().try_into()?;
    Ok(Value::Float(x.sin()))
}
