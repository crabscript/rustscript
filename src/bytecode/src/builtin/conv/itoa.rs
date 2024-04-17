use std::rc::Weak;

use anyhow::Result;

use crate::{FnType, Value, W};

pub const ITOA_SYM: &str = "itoa";

pub fn itoa() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: ITOA_SYM.into(),
        prms: vec!["i".into()],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn itoa_impl(i: &Value) -> Result<Value> {
    let i: i64 = i.clone().try_into()?;
    Ok(Value::String(i.to_string()))
}
