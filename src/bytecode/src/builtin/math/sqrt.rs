use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{Environment, FnType, Value};

pub const SQRT_SYM: &str = "sqrt";

pub fn sqrt(global_env: Rc<RefCell<Environment>>) -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: SQRT_SYM.into(),
        prms: vec!["x".into()],
        addr: 0,
        env: global_env,
    }
}

pub fn sqrt_impl(x: &Value) -> Result<Value> {
    let x: f64 = x.clone().try_into()?;
    Ok(Value::Float(x.sqrt()))
}
