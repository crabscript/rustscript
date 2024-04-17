use std::rc::Weak;

use anyhow::Result;

use crate::{FnType, Value, W};

pub const ATOI_SYM: &str = "atoi";

pub fn atoi() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: ATOI_SYM.into(),
        prms: vec!["s".into()],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn atoi_impl(s: &Value) -> Result<Value> {
    let s: String = s.clone().try_into()?;
    let n: i64 = s.parse()?;
    Ok(Value::Int(n))
}
