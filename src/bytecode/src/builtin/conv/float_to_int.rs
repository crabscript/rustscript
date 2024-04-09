use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{Environment, FnType, Value, W};

pub const FLOAT_TO_INT_SYM: &str = "float_to_int";

pub fn float_to_int(global_env: Rc<RefCell<Environment>>) -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: FLOAT_TO_INT_SYM.into(),
        prms: vec!["x".into()],
        addr: 0,
        env: W(global_env),
    }
}

pub fn float_to_int_impl(x: &Value) -> Result<Value> {
    let x: f64 = x.clone().try_into()?;
    Ok(Value::Int(x as i64))
}
