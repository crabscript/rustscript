use std::{cell::RefCell, rc::Rc};

use crate::{Environment, FnType, Value};

pub const PRINTLN_SYM: &str = "println";

pub fn println(global_env: Rc<RefCell<Environment>>) -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: PRINTLN_SYM.into(),
        prms: vec!["s".into()],
        addr: 0,
        env: global_env,
    }
}

pub fn println_impl(v: &Value) {
    match v {
        Value::Unitialized => println!("uninitialized"),
        Value::Unit => println!("()"),
        Value::String(s) => println!("{}", s),
        Value::Bool(b) => println!("{}", b),
        Value::Int(i) => println!("{}", i),
        Value::Float(f) => println!("{}", f),
        Value::Semaphore(_) => println!("semaphore"),
        Value::Closure { .. } => println!("closure"),
    }
}
