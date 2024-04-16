use std::rc::Weak;

use anyhow::Result;

use crate::{FnType, Value, W};

pub const LOG_SYM: &str = "log";

pub fn log() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: LOG_SYM.into(),
        prms: vec!["x".into()],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn log_impl(x: &Value) -> Result<Value> {
    let x: f64 = x.clone().try_into()?;
    Ok(Value::Float(x.log(10.0)))
}
