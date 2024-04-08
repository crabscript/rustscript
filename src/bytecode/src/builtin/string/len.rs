use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{Environment, FnType, Value, W};

pub const STRING_LEN_SYM: &str = "string_len";

pub fn string_len(global_env: Rc<RefCell<Environment>>) -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: STRING_LEN_SYM.into(),
        prms: vec!["s".into()],
        addr: 0,
        env: W(global_env),
    }
}

pub fn string_len_impl(s: &Value) -> Result<usize> {
    let s: String = s.clone().try_into()?;
    Ok(s.len())
}
