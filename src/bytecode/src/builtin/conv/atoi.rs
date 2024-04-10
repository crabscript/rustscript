use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{Environment, FnType, Value};

pub const ATOI_SYM: &str = "atoi";

pub fn atoi(global_env: Rc<RefCell<Environment>>) -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: ATOI_SYM.into(),
        prms: vec!["s".into()],
        addr: 0,
        env: global_env,
    }
}

pub fn atoi_impl(s: &Value) -> Result<Value> {
    let s: String = s.clone().try_into()?;
    let n: i64 = s.parse()?;
    Ok(Value::Int(n))
}
