use std::rc::Weak;

use crate::{FnType, Value, W};

pub const PRINT_SYM: &str = "print";

pub fn print() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: PRINT_SYM.into(),
        prms: vec!["s".into()],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn print_impl(v: &Value) {
    match v {
        Value::Unitialized => print!("uninitialized"),
        Value::Unit => print!("()"),
        Value::String(s) => print!("{}", s),
        Value::Bool(b) => print!("{}", b),
        Value::Int(i) => print!("{}", i),
        Value::Float(f) => print!("{}", f),
        Value::Semaphore(_) => print!("semaphore"),
        Value::Closure { .. } => print!("closure"),
    }
}
