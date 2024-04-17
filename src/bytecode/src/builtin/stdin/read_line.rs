use std::rc::Weak;

use anyhow::Result;

use crate::{FnType, Value, W};

pub const READ_LINE_SYM: &str = "read_line";

pub fn read_line() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: READ_LINE_SYM.into(),
        prms: vec![],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn read_line_impl() -> Result<String> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input)
}
