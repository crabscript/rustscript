use std::rc::Weak;

use anyhow::Result;

use crate::{FnType, Value, W};

pub const COS_SYM: &str = "cos";

pub fn cos() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: COS_SYM.into(),
        prms: vec!["x".into()],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn cos_impl(x: &Value) -> Result<Value> {
    let x: f64 = x.clone().try_into()?;
    Ok(Value::Float(x.cos()))
}
