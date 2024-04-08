use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{Environment, FnType, Value, W};

pub const READ_LINE_SYM: &str = "read_line";

pub fn read_line(global_env: Rc<RefCell<Environment>>) -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: READ_LINE_SYM.into(),
        prms: vec![],
        addr: 0,
        env: W(global_env),
    }
}

pub fn read_line_impl() -> Result<String> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input)
}
