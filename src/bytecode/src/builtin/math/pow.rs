use std::rc::Weak;

use anyhow::Result;

use crate::{FnType, Value, W};

pub const POW_SYM: &str = "pow";

pub fn pow() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: POW_SYM.into(),
        prms: vec!["base".into(), "exp".into()],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn pow_impl(base: &Value, exp: &Value) -> Result<Value> {
    let base: f64 = base.clone().try_into()?;
    let exp: f64 = exp.clone().try_into()?;
    Ok(Value::Float(base.powf(exp)))
}
