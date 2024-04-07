use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{Environment, FnType, Value, W};

pub const LOG_SYM: &str = "log";

pub fn log(global_env: Rc<RefCell<Environment>>) -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: LOG_SYM.into(),
        prms: vec!["x".into()],
        addr: 0,
        env: W(global_env),
    }
}

pub fn log_impl(x: &Value) -> Result<Value> {
    let x: f64 = x.clone().try_into()?;
    Ok(Value::Float(x.log(10.0)))
}
