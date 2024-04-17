use std::rc::Weak;

use anyhow::Result;

use crate::{FnType, Value, W};

pub const STRING_LEN_SYM: &str = "string_len";

pub fn string_len() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: STRING_LEN_SYM.into(),
        prms: vec!["s".into()],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn string_len_impl(s: &Value) -> Result<usize> {
    let s: String = s.clone().try_into()?;
    Ok(s.len())
}
