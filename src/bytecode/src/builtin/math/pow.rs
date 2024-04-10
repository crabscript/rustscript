use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{Environment, FnType, Value};

pub const POW_SYM: &str = "pow";

pub fn pow(global_env: Rc<RefCell<Environment>>) -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: POW_SYM.into(),
        prms: vec!["base".into(), "exp".into()],
        addr: 0,
        env: global_env,
    }
}

pub fn pow_impl(base: &Value, exp: &Value) -> Result<Value> {
    let base: f64 = base.clone().try_into()?;
    let exp: f64 = exp.clone().try_into()?;
    Ok(Value::Float(base.powf(exp)))
}
