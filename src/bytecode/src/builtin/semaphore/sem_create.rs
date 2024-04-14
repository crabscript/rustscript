use std::{cell::RefCell, rc::Rc};

use crate::{Environment, FnType, Semaphore, Value};

pub const SEM_CREATE_SYM: &str = "sem_create";

pub fn sem_create(global_env: Rc<RefCell<Environment>>) -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: SEM_CREATE_SYM.into(),
        prms: vec![],
        addr: 0,
        env: global_env,
    }
}

pub fn sem_create_impl() -> Value {
    Semaphore::default().into()
}
