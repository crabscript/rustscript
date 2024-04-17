use std::rc::Weak;

use crate::{FnType, Semaphore, Value, W};

pub const SEM_CREATE_SYM: &str = "sem_create";

pub fn sem_create() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: SEM_CREATE_SYM.into(),
        prms: vec![],
        addr: 0,
        env: W(Weak::new()),
    }
}

pub fn sem_create_impl() -> Value {
    Semaphore::default().into()
}
