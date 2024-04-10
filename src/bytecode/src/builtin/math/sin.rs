use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{Environment, FnType, Value};

pub const SIN_SYM: &str = "sin";

pub fn sin(global_env: Rc<RefCell<Environment>>) -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: SIN_SYM.into(),
        prms: vec!["x".into()],
        addr: 0,
        env: global_env,
    }
}

pub fn sin_impl(x: &Value) -> Result<Value> {
    let x: f64 = x.clone().try_into()?;
    Ok(Value::Float(x.sin()))
}
