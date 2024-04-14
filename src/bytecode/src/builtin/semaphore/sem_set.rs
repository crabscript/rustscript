use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::{Environment, FnType, Semaphore, Value};

pub const SEM_SET_SYM: &str = "sem_set";

pub fn sem_set(global_env: Rc<RefCell<Environment>>) -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: SEM_SET_SYM.into(),
        prms: vec![],
        addr: 2,
        env: global_env,
    }
}

pub fn sem_set_impl(sem: &Value, val: &Value) -> Result<()> {
    let sem: Semaphore = sem.clone().try_into()?;
    let val: i64 = val.clone().try_into()?;

    let mut sem_guard = sem.lock().unwrap();
    *sem_guard = val as u64;

    Ok(())
}
