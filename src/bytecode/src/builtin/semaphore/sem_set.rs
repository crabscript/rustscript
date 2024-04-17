use std::rc::Weak;

use anyhow::Result;

use crate::{FnType, Semaphore, Value, W};

pub const SEM_SET_SYM: &str = "sem_set";

pub fn sem_set() -> Value {
    Value::Closure {
        fn_type: FnType::Builtin,
        sym: SEM_SET_SYM.into(),
        prms: vec![],
        addr: 2,
        env: W(Weak::new()),
    }
}

pub fn sem_set_impl(sem: &Value, val: &Value) -> Result<()> {
    let sem: Semaphore = sem.clone().try_into()?;
    let val: i64 = val.clone().try_into()?;

    let mut sem_guard = sem.lock().unwrap();
    *sem_guard = val as u64;

    Ok(())
}
