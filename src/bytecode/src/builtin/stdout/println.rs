use std::rc::Weak;

use crate::{FnType, Value, W};

pub const PRINTLN_SYM: &str = "println";

pub fn println() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: PRINTLN_SYM.into(),
        prms: vec!["s".into()],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn println_impl(v: &Value) {
    println!("{v}");
}
