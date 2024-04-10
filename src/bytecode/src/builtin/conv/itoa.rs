use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{Environment, FnType, Value};

pub const ITOA_SYM: &str = "itoa";

pub fn itoa(global_env: Rc<RefCell<Environment>>) -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: ITOA_SYM.into(),
        prms: vec!["i".into()],
        addr: 0,
        env: global_env,
    }
}

pub fn itoa_impl(i: &Value) -> Result<Value> {
    let i: i64 = i.clone().try_into()?;
    Ok(Value::String(i.to_string()))
}
