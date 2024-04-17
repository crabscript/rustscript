use std::rc::Weak;

use anyhow::Result;

use crate::{FnType, Value, W};

pub const SQRT_SYM: &str = "sqrt";

pub fn sqrt() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: SQRT_SYM.into(),
        prms: vec!["x".into()],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn sqrt_impl(x: &Value) -> Result<Value> {
    let x: f64 = x.clone().try_into()?;
    Ok(Value::Float(x.sqrt()))
}
